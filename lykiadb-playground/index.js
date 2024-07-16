import * as wasm from "./pkg/lykiadb_playground";
console.time("parse");
for (let i = 0; i < 100000; i++) {
  wasm.parse("SELECT * FROM foo;");
}
console.timeEnd("parse");

console.log(wasm.parse("SELECT * FROM foo;"));
