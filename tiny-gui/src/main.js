
const { invoke, Channel } = window.__TAURI__.core;

// let greetInputEl;
// let greetMsgEl;

// async function greet() {
//   // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
//   greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
// }

let userInputEle;
let outputEle;

async function chat(user_message) {
  console.log("userinput " + user_message)
  const onEvent = new Channel();

  onEvent.onmessage = (message) => {
    outputEle.textContent += message;
  };

  await invoke("chat", { "message": user_message, "onChunk": onEvent });
}

window.addEventListener("DOMContentLoaded", () => {
  // greetInputEl = document.querySelector("#greet-input");
  // greetMsgEl = document.querySelector("#greet-msg");
  // document.querySelector("#greet-form").addEventListener("submit", (e) => {
  //   e.preventDefault();
  //   greet();
  // });

  userInputEle = document.querySelector("#user-input");
  outputEle = document.querySelector("#output");

  document.querySelector("#chat-form").addEventListener("submit", (e)=>{
    e.preventDefault();

    const message = userInputEle.value;

    userInputEle.value = "";

    // TODO: need manage the button state to prevent user send before current round completed
    chat(message);
  });
});
