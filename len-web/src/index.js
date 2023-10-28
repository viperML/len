// import init, { inc, lexer } from "../pkg/index.js";
import { initFlowbite } from "flowbite";

import * as crate from "../pkg/index";
import "./style.css";

async function main() {
  const input = document.getElementById("input").value;
  const res = crate.main(input);
  // document.getElementById("output").textContent = res;
  // for all element with name output, set textContent to res
  document.getElementsByName("lexer").forEach((e) => {
    e.textContent = res.lexer;
  });
  document.getElementsByName("ast").forEach((e) => {
    e.textContent = res.ast;
  });
}

// Execute on each keypress of the text input
document.getElementById("input").addEventListener("input", () => {
  main();
});

initFlowbite();
console.log("WASM initialized");
main();

export default {};
