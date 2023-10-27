// import init, { inc, lexer } from "../pkg/index.js";
import * as crate from "../pkg/index.js";
import { initFlowbite } from "flowbite";
import "./style.css" assert { type: 'css' };

async function main() {
  const input = document.getElementById("input").value;
  const res = crate.lexer(input);
  // document.getElementById("output").textContent = res;
  // for all element with name output, set textContent to res
  document.getElementsByName("lexer").forEach((e) => {
    e.textContent = res;
  });
}


// Execute on each keypress of the text input
document.getElementById("input").addEventListener("input", () => {
    main();
});

initFlowbite();
console.log("WASM initialized");
main();

// document.getElementById("submit").addEventListener("click", () => {
//     console.log("submit");
//     main();
// })

export default {};
