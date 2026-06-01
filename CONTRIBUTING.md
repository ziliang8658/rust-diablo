# Contributing

Thanks for considering a contribution to Rust Diablo. This project is primarily a Rust learning and engine-research project, so clear explanations, small changes, and reproducible tests are preferred.

## Ground Rules

- Do not commit or attach proprietary Diablo assets, including `Diabdat.mpq`, extracted sprites, palette dumps, audio files, or asset-derived fixtures.
- Keep changes focused. A parser fix, a rendering bug fix, or a documentation cleanup should usually be its own pull request.
- Prefer tests for binary parsers and data transformations.
- Document behavior that was learned by comparing against Diablo-compatible formats or DevilutionX.
- Avoid broad rewrites unless an issue or design note already explains why the rewrite is needed.

## Local Setup

Install Rust and SDL2 development libraries, then run:

```powershell
cargo build
cargo test
```

For asset-backed examples, place your legally owned `Diabdat.mpq` in `assets/Diabdat.mpq`. Asset-backed tests should either be ignored by default or clearly document the local file requirement.

## Pull Request Checklist

- The change is scoped to one topic.
- `cargo fmt` has been run.
- `cargo test` or relevant targeted tests have been run.
- New binary format behavior includes tests or a short explanation of why tests are not practical yet.
- No proprietary assets or generated asset dumps are included.
- Documentation links remain valid.

## Useful Areas To Help

- Parser tests for MPQ, PCX, CLX, CL2, CEL, TRN, MIN, TIL, and SOL.
- Safer error handling around malformed binary data.
- Rendering validation and visual comparison tooling.
- Documentation cleanup and English/Chinese summaries.
- CI improvements that can run without proprietary assets.

