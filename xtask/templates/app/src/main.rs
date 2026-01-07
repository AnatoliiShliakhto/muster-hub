#[mhub_runtime::main(memory_efficient)]
async fn main() -> Result<()> {
    {{project-name | snake_case}}::run()
}