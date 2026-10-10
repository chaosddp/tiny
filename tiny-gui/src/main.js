
const { invoke, Channel } = window.__TAURI__.core;


let submitBtn;
let userInputEle;
let messageContainer;

async function chat(user_message) {
  Alpine.store("session").add_user(user_message);

  const onEvent = new Channel();

  onEvent.onmessage = (chunk) => {
    console.log(chunk);

    Alpine.store("session").add(chunk);

    if (chunk.finish_reason != null) {
      submitBtn.disabled = false;
    }
  };

  await invoke("chat", { "message": user_message, "onChunk": onEvent });
}

document.addEventListener('alpine:init', () => {
  Alpine.store("session", {
    "messages": [],

    add(chunk) {
      if (this.messages.length == 0) {
        this.messages.push({ "role": "assistant", "id": chunk.id, "content": chunk.content, "reasoning": chunk.reasoning });
      } else {
        let lastMessage = this.messages[this.messages.length - 1];

        if (lastMessage.id == chunk.id) {
          // do update
          if (chunk.content != null) {
            lastMessage.content += chunk.content;
          }

          if (chunk.reasoning != null) {
            lastMessage.reasoning += chunk.reasoning;
          }
        } else {
          // a new one
          this.messages.push({ "role": "assistant", "id": chunk.id, "content": chunk.content, "reasoning": chunk.reasoning });
        }
      }

      messageContainer.scrollTop = messageContainer.scrollHeight
    },
    add_user(msg) {
      this.messages.push({ "role": "user", "id": Math.floor(Date.now() / 1000), "content": msg, "reasoning": null });
      messageContainer.scrollTop = messageContainer.scrollHeight
    }
  });
});

window.addEventListener("DOMContentLoaded", () => {
  submitBtn = document.querySelector("#submit-btn");
  userInputEle = document.querySelector("#user-input");
  messageContainer = document.querySelector("#message-container");

  document.querySelector("#chat-form").addEventListener("submit", (e) => {
    e.preventDefault();

    const message = userInputEle.value;

    userInputEle.value = "";

    submitBtn.disabled = true;

    chat(message);
  });


});
