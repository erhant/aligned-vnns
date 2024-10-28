use sp1_helper::build_program_with_args;

fn main() {
    let args = sp1_helper::BuildArgs {
        elf_name: "riscv32im-succinct-vnns-elf".to_string(),
        ..Default::default()
    };
    build_program_with_args("../program", args)
}
