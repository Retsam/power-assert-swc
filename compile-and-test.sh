set -x
cargo build --release --target wasm32-wasip1
cd js-test-proj
npm i
npm run generate-output
