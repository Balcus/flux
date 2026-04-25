pub trait Command {
    type Output;
    fn run(&mut self) -> anyhow::Result<Self::Output>;
}
