pub trait Command {
    fn run(&mut self) -> anyhow::Result<()>;
}
