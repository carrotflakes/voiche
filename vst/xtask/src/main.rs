fn main() -> nih_plug_xtask::Result<()> {
    // nih_plug_xtask::main();

    let mut args = std::env::args().skip(1);
    // chdir_workspace_root()?;

    let usage_string = "N/A";
    let command = args.next().unwrap();
    let packages = [args.next().unwrap()];
    let other_args = args.collect::<Vec<_>>();
    match command.as_str() {
        "bundle" => {
            // As explained above, for efficiency's sake this is a two step process
            nih_plug_xtask::build(&packages, &other_args)?;

            nih_plug_xtask::bundle(&packages[0], &other_args, false)?;

            Ok(())
        }
        "bundle-universal" => {
            // The same as `--bundle`, but builds universal binaries for macOS Cargo will also error
            // out on duplicate `--target` options, but it seems like a good idea to preemptively
            // abort the bundling process if that happens

            for arg in &other_args {
                if arg == "--target" || arg.starts_with("--target=") {
                    panic!()
                }
            }

            // We can just use the regular build function here. There's sadly no way to build both
            // targets in parallel, so this will likely take twice as logn as a regular build.
            // TODO: Explicitly specifying the target even on the native target causes a rebuild in
            //       the target `target/<target_triple>` directory. This makes bundling much simpler
            //       because there's no conditional logic required based on the current platform,
            //       but it does waste some resources and requires a rebuild if the native target
            //       was already built.
            let mut x86_64_args = other_args.clone();
            x86_64_args.push(String::from("--target=x86_64-apple-darwin"));
            nih_plug_xtask::build(&packages, &x86_64_args)?;
            let mut aarch64_args = other_args.clone();
            aarch64_args.push(String::from("--target=aarch64-apple-darwin"));
            nih_plug_xtask::build(&packages, &aarch64_args)?;

            // This `true` indicates a universal build. This will cause the two sets of built
            // binaries to beq lipo'd together into universal binaries before bundling
            nih_plug_xtask::bundle(&packages[0], &other_args, true)?;

            Ok(())
        }
        // This is only meant to be used by the CI, since using awk for this can be a bit spotty on
        // macOS
        "known-packages" => nih_plug_xtask::list_known_packages(),
        _ => panic!("Unknown command '{command}'\n\n{usage_string}"),
    }
}
