use super::engine::{
    MatrixCard, MatrixConditions, MatrixMode, MatrixPokemon, MatrixResolver, SpeedOutcome,
    build_cards, final_stats,
};
use super::showdown::parse_showdown_team;
use crate::error::PcError;
use crate::model::{BattleSide, Room, StatPoints, Status, Terrain, Weather};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
};
use std::fs;
use std::io::{self, Stdout};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub(super) fn run(team_path: &Path, opponents_path: &Path) -> Result<(), PcError> {
    let mut app = MatrixApp::load(team_path, opponents_path)?;
    enable_raw_mode().map_err(terminal_error)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(terminal_error)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout)).map_err(terminal_error)?;

    let result = run_loop(&mut terminal, &mut app);

    disable_raw_mode().map_err(terminal_error)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(terminal_error)?;
    terminal.show_cursor().map_err(terminal_error)?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut MatrixApp,
) -> Result<(), PcError> {
    loop {
        terminal
            .draw(|frame| render(frame, app))
            .map_err(terminal_error)?;
        if event::poll(Duration::from_millis(200)).map_err(terminal_error)? {
            let Event::Key(key) = event::read().map_err(terminal_error)? else {
                continue;
            };
            if app.handle_key(key)? {
                return Ok(());
            }
        }
    }
}

fn terminal_error(source: io::Error) -> PcError {
    PcError::Validation {
        message: format!("terminal error: {source}"),
    }
}

struct MatrixApp {
    team_path: PathBuf,
    opponents_path: PathBuf,
    player: Vec<MatrixPokemon>,
    opponents: Vec<MatrixPokemon>,
    conditions: MatrixConditions,
    mode: MatrixMode,
    selected_player: usize,
    selected_opponent: usize,
    scroll_row: usize,
    grid_cols: usize,
    visible_rows: usize,
    show_help: bool,
    show_conditions: bool,
    search_active: bool,
    search_query: String,
    status: String,
    cards: Vec<MatrixCard>,
}

impl MatrixApp {
    fn load(team_path: &Path, opponents_path: &Path) -> Result<Self, PcError> {
        let mut app = Self {
            team_path: team_path.to_path_buf(),
            opponents_path: opponents_path.to_path_buf(),
            player: Vec::new(),
            opponents: Vec::new(),
            conditions: MatrixConditions::with_sizes(0, 0),
            mode: MatrixMode::Offensive,
            selected_player: 0,
            selected_opponent: 0,
            scroll_row: 0,
            grid_cols: 1,
            visible_rows: 1,
            show_help: false,
            show_conditions: false,
            search_active: false,
            search_query: String::new(),
            status: String::new(),
            cards: Vec::new(),
        };
        app.reload()?;
        Ok(app)
    }

    fn reload(&mut self) -> Result<(), PcError> {
        let team_text = read_file(&self.team_path)?;
        let opponents_text = read_file(&self.opponents_path)?;
        let player_sets = parse_showdown_team(&team_text)?;
        let opponent_sets = parse_showdown_team(&opponents_text)?;
        let mut resolver = MatrixResolver::default();
        self.player = resolver.resolve_team(&player_sets)?;
        self.opponents = resolver.resolve_team(&opponent_sets)?;
        self.conditions
            .resize(self.player.len(), self.opponents.len());
        self.selected_player = self
            .selected_player
            .min(self.player.len().saturating_sub(1));
        self.selected_opponent = self
            .selected_opponent
            .min(self.opponents.len().saturating_sub(1));
        self.recompute()?;
        self.status = format!(
            "Loaded {} team Pokemon and {} opponents",
            self.player.len(),
            self.opponents.len()
        );
        Ok(())
    }

    fn recompute(&mut self) -> Result<(), PcError> {
        if self.player.is_empty() || self.opponents.is_empty() {
            self.cards.clear();
            return Ok(());
        }
        self.cards = build_cards(
            self.mode,
            &self.player[self.selected_player],
            self.selected_player,
            &self.opponents,
            &self.conditions,
        )?;
        self.ensure_selected_visible();
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool, PcError> {
        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') => self.show_help = false,
                _ => {}
            }
            return Ok(false);
        }
        if self.show_conditions {
            return self.handle_conditions_key(key);
        }
        if self.search_active {
            self.handle_search_key(key);
            return Ok(false);
        }
        if matches!(key.code, KeyCode::Char('q')) {
            return Ok(true);
        }

        match key.code {
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('c') => self.show_conditions = true,
            KeyCode::Char('/') => self.start_search(),
            KeyCode::Char('r') => {
                if let Err(error) = self.reload() {
                    self.status = error.to_string();
                }
            }
            KeyCode::Char('1') => self.set_mode(MatrixMode::Offensive)?,
            KeyCode::Char('2') => self.set_mode(MatrixMode::Defensive)?,
            KeyCode::Char('3') => self.set_mode(MatrixMode::Speed)?,
            KeyCode::Up | KeyCode::Char('k') => self.select_player_delta(-1)?,
            KeyCode::Down | KeyCode::Char('j') => self.select_player_delta(1)?,
            KeyCode::Left | KeyCode::Char('h') => self.select_opponent_delta(-1),
            KeyCode::Right | KeyCode::Char('l') => self.select_opponent_delta(1),
            KeyCode::PageUp => self.select_opponent_delta(-(self.page_size() as isize)),
            KeyCode::PageDown => self.select_opponent_delta(self.page_size() as isize),
            _ => {}
        }
        Ok(false)
    }

    fn handle_conditions_key(&mut self, key: KeyEvent) -> Result<bool, PcError> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('c') => self.show_conditions = false,
            KeyCode::Char('w') => {
                self.conditions.field.weather = next_weather(self.conditions.field.weather)
            }
            KeyCode::Char('t') => {
                self.conditions.field.terrain = next_terrain(self.conditions.field.terrain)
            }
            KeyCode::Char('m') => {
                self.conditions.field.room = if self.conditions.field.room == Room::TrickRoom {
                    Room::None
                } else {
                    Room::TrickRoom
                };
            }
            KeyCode::Char('a') => {
                if let Some(status) = self
                    .conditions
                    .player_statuses
                    .get_mut(self.selected_player)
                {
                    *status = next_status(*status);
                }
            }
            KeyCode::Char('e') => {
                if let Some(status) = self
                    .conditions
                    .opponent_statuses
                    .get_mut(self.selected_opponent)
                {
                    *status = next_status(*status);
                }
            }
            KeyCode::Char('1') => {
                self.conditions.player_side.tailwind = !self.conditions.player_side.tailwind
            }
            KeyCode::Char('2') => {
                self.conditions.player_side.reflect = !self.conditions.player_side.reflect
            }
            KeyCode::Char('3') => {
                self.conditions.player_side.light_screen = !self.conditions.player_side.light_screen
            }
            KeyCode::Char('4') => {
                self.conditions.player_side.aurora_veil = !self.conditions.player_side.aurora_veil
            }
            KeyCode::Char('5') => {
                self.conditions.player_side.helping_hand = !self.conditions.player_side.helping_hand
            }
            KeyCode::Char('6') => {
                self.conditions.opponent_side.tailwind = !self.conditions.opponent_side.tailwind
            }
            KeyCode::Char('7') => {
                self.conditions.opponent_side.reflect = !self.conditions.opponent_side.reflect
            }
            KeyCode::Char('8') => {
                self.conditions.opponent_side.light_screen =
                    !self.conditions.opponent_side.light_screen
            }
            KeyCode::Char('9') => {
                self.conditions.opponent_side.aurora_veil =
                    !self.conditions.opponent_side.aurora_veil
            }
            KeyCode::Char('0') => {
                self.conditions.opponent_side.helping_hand =
                    !self.conditions.opponent_side.helping_hand
            }
            _ => {}
        }
        self.recompute()?;
        Ok(false)
    }

    fn start_search(&mut self) {
        self.search_active = true;
        self.search_query.clear();
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search_active = false;
                self.status = "Search closed".to_string();
            }
            KeyCode::Enter => {
                self.search_active = false;
                if let Some(opponent) = self.opponents.get(self.selected_opponent) {
                    self.status = format!("Selected {}", opponent.set.display_name());
                }
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.select_search_match();
            }
            KeyCode::Char(value)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.search_query.push(value);
                self.select_search_match();
            }
            _ => {}
        }
    }

    fn set_mode(&mut self, mode: MatrixMode) -> Result<(), PcError> {
        self.mode = mode;
        self.recompute()
    }

    fn select_player_delta(&mut self, delta: isize) -> Result<(), PcError> {
        if self.player.is_empty() {
            return Ok(());
        }
        self.selected_player = offset_index(self.selected_player, delta, self.player.len());
        self.recompute()
    }

    fn select_opponent_delta(&mut self, delta: isize) {
        if self.opponents.is_empty() {
            return;
        }
        self.selected_opponent = offset_index(self.selected_opponent, delta, self.opponents.len());
        self.ensure_selected_visible();
    }

    fn ensure_selected_visible(&mut self) {
        if self.grid_cols == 0 || self.visible_rows == 0 {
            return;
        }
        let selected_row = self.selected_opponent / self.grid_cols;
        if selected_row < self.scroll_row {
            self.scroll_row = selected_row;
        } else if selected_row >= self.scroll_row + self.visible_rows {
            self.scroll_row = selected_row + 1 - self.visible_rows;
        }
    }

    fn page_size(&self) -> usize {
        (self.grid_cols * self.visible_rows).max(1)
    }

    fn select_search_match(&mut self) {
        if let Some(index) = opponent_search_matches(&self.opponents, &self.search_query)
            .first()
            .copied()
        {
            self.selected_opponent = index;
            self.ensure_selected_visible();
        }
    }
}

fn read_file(path: &Path) -> Result<String, PcError> {
    fs::read_to_string(path).map_err(|source| PcError::Io {
        path: path.display().to_string(),
        source,
    })
}

fn offset_index(current: usize, delta: isize, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let next = current as isize + delta;
    next.clamp(0, len.saturating_sub(1) as isize) as usize
}

fn render(frame: &mut Frame<'_>, app: &mut MatrixApp) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .split(area);
    render_top(frame, chunks[0], app);
    render_body(frame, chunks[1], app);
    render_status(frame, chunks[2], app);
    if app.show_help {
        render_help(frame, centered(area, 86, 15));
    }
    if app.show_conditions {
        render_conditions(frame, centered(area, 96, 28), app);
    }
}

fn render_top(frame: &mut Frame<'_>, area: Rect, app: &MatrixApp) {
    let mode = format!(
        "[1] {}  [2] {}  [3] {}",
        mode_label(app.mode, MatrixMode::Offensive),
        mode_label(app.mode, MatrixMode::Defensive),
        mode_label(app.mode, MatrixMode::Speed)
    );
    let conditions = format!(
        "Weather {} | Terrain {} | Room {}",
        app.conditions.field.weather, app.conditions.field.terrain, app.conditions.field.room
    );
    let title = if let Some(player) = app.player.get(app.selected_player) {
        format!("Matrix - {} vs Meta", player.set.display_name())
    } else {
        "Matrix".to_string()
    };
    let text = Text::from(vec![
        Line::from(vec![
            Span::styled(
                title,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::raw(mode),
        ]),
        Line::from(vec![
            Span::styled(conditions, Style::default().fg(Color::Cyan)),
            Span::raw("    / search  c conditions  r reload  ? help  q quit"),
        ]),
    ]);
    frame.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::BOTTOM)),
        area,
    );
}

fn mode_label(active: MatrixMode, mode: MatrixMode) -> String {
    if active == mode {
        format!("*{}*", mode.title())
    } else {
        mode.title().to_string()
    }
}

fn render_body(frame: &mut Frame<'_>, area: Rect, app: &mut MatrixApp) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(34), Constraint::Min(30)])
        .split(area);
    render_team(frame, chunks[0], app);
    render_matrix(frame, chunks[1], app);
}

fn render_team(frame: &mut Frame<'_>, area: Rect, app: &MatrixApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(8)])
        .split(area);
    let items = app
        .player
        .iter()
        .enumerate()
        .map(|(index, pokemon)| {
            let prefix = if index == app.selected_player {
                "> "
            } else {
                "  "
            };
            let text = format!(
                "{}{} @ {}",
                prefix,
                pokemon.set.display_name(),
                pokemon.set.item.as_deref().unwrap_or("-")
            );
            let style = if index == app.selected_player {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default()
            };
            ListItem::new(text).style(style)
        })
        .collect::<Vec<_>>();
    let mut list_state = ListState::default().with_selected(Some(app.selected_player));
    frame.render_stateful_widget(
        List::new(items).block(
            Block::default()
                .title(format!("Your team ({})", app.player.len()))
                .borders(Borders::ALL),
        ),
        chunks[0],
        &mut list_state,
    );

    let Some(player) = app.player.get(app.selected_player) else {
        frame.render_widget(
            Paragraph::new("No Pokemon loaded")
                .block(Block::default().title("Details").borders(Borders::ALL)),
            chunks[1],
        );
        return;
    };
    let stats = final_stats(player, BattleSide::Player);
    let lines = vec![
        Line::from(vec![Span::styled(
            player.set.display_name().to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled(
                player.set.item.as_deref().unwrap_or("-").to_string(),
                Style::default().fg(Color::Gray),
            ),
            Span::raw(" | "),
            Span::styled(
                player.set.ability.as_deref().unwrap_or("-").to_string(),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(format!(
            "{} Lv{} | HP {}",
            player.set.nature, player.set.level, stats.hp
        )),
        Line::from(format!(
            "Atk {}  Def {}  SpA {}",
            stats.atk, stats.def, stats.spa
        )),
        Line::from(format!("SpD {}  Spe {}", stats.spd, stats.spe)),
        Line::from(Span::styled(
            short_sps(player.set.stat_points),
            Style::default().fg(Color::DarkGray),
        )),
    ];
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }).block(
            Block::default()
                .title(" Selected ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        ),
        chunks[1],
    );
}

fn render_matrix(frame: &mut Frame<'_>, area: Rect, app: &mut MatrixApp) {
    let block = Block::default()
        .title(format!("{} matrix", app.mode.title()))
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.player.is_empty() {
        frame.render_widget(centered_message("No team Pokemon loaded."), inner);
        return;
    }
    if app.opponents.is_empty() {
        frame.render_widget(
            centered_message("No opponents loaded. Add Showdown sets to examples/opponents.txt."),
            inner,
        );
        return;
    }

    app.grid_cols = if inner.width >= 128 {
        4
    } else if inner.width >= 96 {
        3
    } else if inner.width >= 64 {
        2
    } else {
        1
    };
    let card_height = 7u16;
    app.visible_rows = (inner.height / card_height).max(1) as usize;
    app.ensure_selected_visible();

    let row_constraints = vec![Constraint::Length(card_height); app.visible_rows];
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(inner);
    let start = app.scroll_row * app.grid_cols;

    for (row_index, row_area) in rows.iter().enumerate() {
        let col_constraints = vec![Constraint::Ratio(1, app.grid_cols as u32); app.grid_cols];
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .split(*row_area);
        for (col_index, card_area) in cols.iter().enumerate() {
            let index = start + row_index * app.grid_cols + col_index;
            if let Some(card) = app.cards.get(index) {
                render_card(frame, *card_area, app, card);
            }
        }
    }
}

fn render_card(frame: &mut Frame<'_>, area: Rect, app: &MatrixApp, card: &MatrixCard) {
    let selected = card.opponent_index == app.selected_opponent;
    let border_style = if selected {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let title = format!("{} {}", type_text(&card.types), card.name);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);
    let mut lines = Vec::new();
    lines.push(Line::from(format!(
        "{} | {}",
        card.item.as_deref().unwrap_or("-"),
        card.ability.as_deref().unwrap_or("-")
    )));
    match app.mode {
        MatrixMode::Speed => {
            if let Some(speed) = &card.speed {
                let outcome_style = speed_outcome_style(speed.outcome);
                lines.push(Line::from(vec![
                    Span::raw("Yours  "),
                    Span::styled(
                        format!("{:>4}", speed.player_speed),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" raw {}", speed.player_raw_speed),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::raw("Theirs "),
                    Span::styled(
                        format!("{:>4}", speed.opponent_speed),
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" raw {}", speed.opponent_raw_speed),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
                lines.push(Line::from(Span::styled(
                    match speed.outcome {
                        SpeedOutcome::PlayerFirst => "You move first",
                        SpeedOutcome::OpponentFirst => "Opponent moves first",
                        SpeedOutcome::Tie => "Speed tie",
                    },
                    outcome_style.add_modifier(Modifier::BOLD),
                )));
            }
        }
        MatrixMode::Offensive | MatrixMode::Defensive => {
            if card.rows.is_empty() {
                lines.push(Line::from("No damaging moves"));
            } else {
                for row in card.rows.iter().take(4) {
                    let damage_style = damage_style(app.mode, row.max_percent);
                    let full_ko = if row.ohko_percent >= 0.5 {
                        format!(" OHKO {:.0}%", row.ohko_percent)
                    } else if row.two_hko_percent >= 0.5 {
                        format!(" 2HKO {:.0}%", row.two_hko_percent)
                    } else {
                        String::new()
                    };
                    let compact_ko = if row.ohko_percent >= 0.5 {
                        " KO".to_string()
                    } else if row.two_hko_percent >= 0.5 {
                        " 2H".to_string()
                    } else {
                        String::new()
                    };
                    let content_width = area.width.saturating_sub(2) as usize;
                    let ko = if content_width >= 42 {
                        full_ko
                    } else {
                        compact_ko
                    };
                    let bar_width = if content_width >= 34 { 10 } else { 7 };
                    let fixed_width = bar_width + 1 + 8 + ko.chars().count();
                    let move_width = content_width.saturating_sub(fixed_width + 1).clamp(7, 12);
                    lines.push(Line::from(vec![
                        Span::raw(format!(
                            "{:<width$} ",
                            truncate(&row.move_name, move_width),
                            width = move_width
                        )),
                        percent_bar(row.max_percent, damage_style, bar_width),
                        Span::raw(" "),
                        Span::styled(
                            format!("{:>3.0}-{:<3.0}%", row.min_percent, row.max_percent),
                            damage_style.add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(ko, damage_style),
                    ]));
                }
            }
        }
    }
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }).block(block),
        area,
    );
}

fn render_status(frame: &mut Frame<'_>, area: Rect, app: &MatrixApp) {
    if app.search_active {
        render_search_status(frame, area, app);
        return;
    }
    frame.render_widget(
        Paragraph::new(app.status.as_str()).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}

fn render_search_status(frame: &mut Frame<'_>, area: Rect, app: &MatrixApp) {
    let matches = opponent_search_matches(&app.opponents, &app.search_query);
    let match_text = if app.search_query.trim().is_empty() {
        "type opponent name".to_string()
    } else if matches.is_empty() {
        "no matches".to_string()
    } else {
        let selected_position = matches
            .iter()
            .position(|index| *index == app.selected_opponent)
            .unwrap_or(0)
            + 1;
        format!("match {selected_position}/{}", matches.len())
    };
    let selected = app
        .opponents
        .get(app.selected_opponent)
        .map(|opponent| opponent.set.display_name())
        .unwrap_or("-");
    let line = Line::from(vec![
        Span::styled("/", key_style()),
        Span::raw(" search opponents: "),
        Span::styled(
            format!("{}_", app.search_query),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(match_text, Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::styled(selected.to_string(), Style::default().fg(Color::Gray)),
        Span::raw("  Enter accept  Esc close"),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_help(frame: &mut Frame<'_>, area: Rect) {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .border_type(BorderType::Rounded);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    "Matrix controls",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    "keyboard-first matchup workbench",
                    Style::default().fg(Color::Gray),
                ),
            ]),
            Line::from(Span::styled(
                "Pick a view, move through cards, and tune battle conditions.",
                Style::default().fg(Color::DarkGray),
            )),
        ]),
        sections[0],
    );

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(sections[1]);

    render_help_panel(
        frame,
        columns[0],
        "Modes",
        Color::Green,
        vec![
            help_line("1", "Offensive damage"),
            help_line("2", "Defensive damage"),
            help_line("3", "Speed matchup"),
        ],
    );
    render_help_panel(
        frame,
        columns[1],
        "Navigate",
        Color::Cyan,
        vec![
            help_line("Up/Dn", "Your Pokemon"),
            help_line("k / j", "Your Pokemon"),
            help_line("Left/Right", "Opponent card"),
            help_line("h / l", "Opponent card"),
            help_line("PgUp/PgDn", "Jump cards"),
            help_line("/", "Search opponents"),
        ],
    );
    render_help_panel(
        frame,
        columns[2],
        "Actions",
        Color::Magenta,
        vec![
            help_line("c", "Conditions"),
            help_line("r", "Reload files"),
            help_line("?", "Help"),
            help_line("q", "Quit"),
        ],
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Esc", key_style()),
            Span::raw(" or "),
            Span::styled("?", key_style()),
            Span::raw(" closes this panel"),
        ]))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray)),
        sections[2],
    );
}

fn render_help_panel(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    color: Color,
    lines: Vec<Line<'static>>,
) {
    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(format!(" {title} "))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color)),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn help_line(key: &str, label: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(key.to_string(), key_style()),
        Span::raw(" "),
        Span::styled(label.to_string(), Style::default().fg(Color::Gray)),
    ])
}

fn render_conditions(frame: &mut Frame<'_>, area: Rect, app: &MatrixApp) {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" Conditions ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .border_type(BorderType::Rounded);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let player_status = app
        .conditions
        .player_statuses
        .get(app.selected_player)
        .copied()
        .unwrap_or(Status::None);
    let opponent_status = app
        .conditions
        .opponent_statuses
        .get(app.selected_opponent)
        .copied()
        .unwrap_or(Status::None);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    "Battle Conditions",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    "press highlighted keys to cycle/toggle",
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Line::from(Span::styled(
                "Weather, terrain, room, status, side screens, Tailwind, and Helping Hand.",
                Style::default().fg(Color::Gray),
            )),
        ]),
        sections[0],
    );

    render_conditions_field(frame, sections[1], app);

    let side_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(sections[2]);
    render_side_conditions(
        frame,
        side_chunks[0],
        "Your Pokemon",
        Color::Green,
        player_status,
        &[
            ("a", "Status", true),
            ("1", "Tailwind", app.conditions.player_side.tailwind),
            ("2", "Reflect", app.conditions.player_side.reflect),
            ("3", "L.Screen", app.conditions.player_side.light_screen),
            ("4", "A.Veil", app.conditions.player_side.aurora_veil),
            ("5", "H.Hand", app.conditions.player_side.helping_hand),
        ],
    );
    render_side_conditions(
        frame,
        side_chunks[1],
        "Enemy Pokemon",
        Color::Red,
        opponent_status,
        &[
            ("e", "Status", true),
            ("6", "Tailwind", app.conditions.opponent_side.tailwind),
            ("7", "Reflect", app.conditions.opponent_side.reflect),
            ("8", "L.Screen", app.conditions.opponent_side.light_screen),
            ("9", "A.Veil", app.conditions.opponent_side.aurora_veil),
            ("0", "H.Hand", app.conditions.opponent_side.helping_hand),
        ],
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Esc", key_style()),
            Span::raw(" / "),
            Span::styled("c", key_style()),
            Span::raw(" close   "),
            Span::styled("1-0", key_style()),
            Span::raw(" toggles   "),
            Span::styled("w/t/m/a/e", key_style()),
            Span::raw(" cycles"),
        ]))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray)),
        sections[3],
    );
}

fn render_conditions_field(frame: &mut Frame<'_>, area: Rect, app: &MatrixApp) {
    let lines = vec![
        Line::from(vec![
            Span::styled("w", key_style()),
            Span::raw(" Weather "),
            value_badge(
                app.conditions.field.weather.to_string(),
                weather_color(app.conditions.field.weather),
            ),
            Span::raw("   "),
            Span::styled("t", key_style()),
            Span::raw(" Terrain "),
            value_badge(app.conditions.field.terrain.to_string(), Color::Cyan),
        ]),
        Line::from(vec![
            Span::styled("m", key_style()),
            Span::raw(" Trick Room "),
            toggle_badge(app.conditions.field.room == Room::TrickRoom),
        ]),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Field ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_side_conditions(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    color: Color,
    status: Status,
    toggles: &[(&str, &str, bool)],
) {
    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(toggles[0].0, key_style()),
        Span::raw(" Status "),
        value_badge(status.to_string(), status_color(status)),
    ]));
    lines.push(Line::from(""));
    for (key, label, enabled) in toggles.iter().skip(1) {
        lines.push(toggle_line(key, label, *enabled));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(format!(" {title} "))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color)),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn centered_message(message: &str) -> Paragraph<'_> {
    Paragraph::new(message)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray))
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(2));
    let height = height.min(area.height.saturating_sub(2));
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

fn short_sps(points: StatPoints) -> String {
    format!(
        "SP HP{} At{} D{} SA{} SD{} Sp{}",
        points.hp, points.atk, points.def, points.spa, points.spd, points.spe
    )
}

fn type_text(types: &[crate::model::PokemonType]) -> String {
    types
        .iter()
        .map(|pokemon_type| pokemon_type.to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn percent_bar(percent: f32, style: Style, width: usize) -> Span<'static> {
    let filled = ((percent.min(100.0) / 100.0) * width as f32).round() as usize;
    let filled = filled.min(width);
    Span::styled(
        format!("{}{}", "#".repeat(filled), "-".repeat(width - filled)),
        style,
    )
}

fn truncate(value: &str, width: usize) -> String {
    if value.chars().count() <= width {
        value.to_string()
    } else {
        value
            .chars()
            .take(width.saturating_sub(1))
            .collect::<String>()
            + "."
    }
}

fn damage_style(mode: MatrixMode, max_percent: f32) -> Style {
    let color = if max_percent >= 100.0 {
        match mode {
            MatrixMode::Offensive => Color::Green,
            MatrixMode::Defensive => Color::Red,
            MatrixMode::Speed => Color::Gray,
        }
    } else if max_percent >= 50.0 {
        Color::Rgb(255, 165, 0)
    } else {
        Color::Gray
    };
    Style::default().fg(color)
}

fn speed_outcome_style(outcome: SpeedOutcome) -> Style {
    match outcome {
        SpeedOutcome::PlayerFirst => Style::default().fg(Color::Green),
        SpeedOutcome::OpponentFirst => Style::default().fg(Color::Red),
        SpeedOutcome::Tie => Style::default().fg(Color::Yellow),
    }
}

fn key_style() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
}

fn value_badge(value: String, color: Color) -> Span<'static> {
    Span::styled(
        format!(" {value} "),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}

fn toggle_badge(enabled: bool) -> Span<'static> {
    if enabled {
        Span::styled(
            " ON ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(" off ", Style::default().fg(Color::DarkGray))
    }
}

fn toggle_line(key: &str, label: &str, enabled: bool) -> Line<'static> {
    Line::from(vec![
        Span::styled(key.to_string(), key_style()),
        Span::raw(format!(" {:<9}", label)),
        toggle_badge(enabled),
    ])
}

fn weather_color(weather: Weather) -> Color {
    match weather {
        Weather::None => Color::Gray,
        Weather::Sun => Color::Yellow,
        Weather::Rain => Color::Blue,
        Weather::Sand => Color::Rgb(194, 154, 83),
        Weather::Snow => Color::Cyan,
    }
}

fn status_color(status: Status) -> Color {
    match status {
        Status::None => Color::Gray,
        Status::Burn => Color::Red,
        Status::Paralysis => Color::Yellow,
        Status::Poison | Status::Toxic => Color::Magenta,
        Status::Sleep => Color::Blue,
        Status::Freeze => Color::Cyan,
    }
}

fn next_weather(weather: Weather) -> Weather {
    match weather {
        Weather::None => Weather::Sun,
        Weather::Sun => Weather::Rain,
        Weather::Rain => Weather::Sand,
        Weather::Sand => Weather::Snow,
        Weather::Snow => Weather::None,
    }
}

fn next_terrain(terrain: Terrain) -> Terrain {
    match terrain {
        Terrain::None => Terrain::Electric,
        Terrain::Electric => Terrain::Grassy,
        Terrain::Grassy => Terrain::Psychic,
        Terrain::Psychic => Terrain::Misty,
        Terrain::Misty => Terrain::None,
    }
}

fn next_status(status: Status) -> Status {
    match status {
        Status::None => Status::Burn,
        Status::Burn => Status::Paralysis,
        Status::Paralysis => Status::Poison,
        Status::Poison => Status::Toxic,
        Status::Toxic => Status::Sleep,
        Status::Sleep => Status::Freeze,
        Status::Freeze => Status::None,
    }
}

fn opponent_search_matches(opponents: &[MatrixPokemon], query: &str) -> Vec<usize> {
    let query_key = search_key(query);
    if query_key.is_empty() {
        return Vec::new();
    }
    opponents
        .iter()
        .enumerate()
        .filter_map(|(index, opponent)| {
            let display_key = search_key(opponent.set.display_name());
            let species_key = search_key(&opponent.set.species);
            (display_key.contains(&query_key) || species_key.contains(&query_key)).then_some(index)
        })
        .collect()
}

fn search_key(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::search_key;

    #[test]
    fn search_key_ignores_spacing_and_punctuation() {
        assert_eq!(search_key("Iron Hands"), "ironhands");
        assert_eq!(search_key("iron-hands"), "ironhands");
        assert_eq!(search_key("IRON_HANDS"), "ironhands");
    }
}
