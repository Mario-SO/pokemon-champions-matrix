fn main() -> miette::Result<()> {
    pc::run().map_err(miette::Report::new)
}
