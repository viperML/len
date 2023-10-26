import init, { inc, lexer } from "./pkg/len_web.js";

async function main() {
    const input = document.getElementById("input").value;
    const res = lexer(input);
    document.getElementById("output").textContent = res;
}


init().then(() => {
    console.log("WASM initialized");
    main();
})

// Execute on each keypress of the text input
document.getElementById("input").addEventListener("input", () => {
    main();
})

// document.getElementById("submit").addEventListener("click", () => {
//     console.log("submit");
//     main();
// })