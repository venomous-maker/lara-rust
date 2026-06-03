use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use crate::command::{Args, Command, CommandMeta};
use anyhow::Result;

type AnyCommand = Arc<dyn AnyCommandExt + Send + Sync>;

#[async_trait]
trait AnyCommandExt: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    async fn run(&self, args: &Args) -> Result<()>;
}

struct CommandWrapper<C: Command>(C);

#[async_trait]
impl<C: Command + 'static> AnyCommandExt for CommandWrapper<C> {
    fn name(&self) -> &'static str { C::command_name() }
    fn description(&self) -> &'static str { C::command_description() }
    async fn run(&self, args: &Args) -> Result<()> { self.0.handle(args).await }
}

/// Console kernel — registers and dispatches commands.
pub struct Kernel {
    commands: HashMap<String, AnyCommand>,
}

impl Kernel {
    pub fn new() -> Self {
        Self { commands: HashMap::new() }
    }

    pub fn register<C: Command + 'static>(mut self, command: C) -> Self {
        let name = C::command_name().to_string();
        self.commands.insert(name, Arc::new(CommandWrapper(command)));
        self
    }

    /// Run a command by name with the given raw argv.
    pub async fn call(&self, name: &str, argv: &[String]) -> Result<()> {
        let args = Args::parse(argv);
        if let Some(cmd) = self.commands.get(name) {
            cmd.run(&args).await
        } else {
            anyhow::bail!("Command `{}` not found. Run `artisan list` to see available commands.", name)
        }
    }

    /// Entry point: reads `std::env::args()` and dispatches.
    pub async fn handle(&self) -> Result<()> {
        let argv: Vec<String> = std::env::args().skip(1).collect();
        if argv.is_empty() {
            self.print_help();
            return Ok(());
        }
        let name = &argv[0];
        self.call(name, &argv[1..]).await
    }

    fn print_help(&self) {
        println!("Lara Artisan {}", env!("CARGO_PKG_VERSION"));
        println!("\nAvailable commands:");
        let mut names: Vec<&str> = self.commands.keys().map(|s| s.as_str()).collect();
        names.sort();
        for name in names {
            let desc = self.commands[name].description();
            println!("  {:30} {}", name, desc);
        }
    }
}

impl Default for Kernel {
    fn default() -> Self { Self::new() }
}
