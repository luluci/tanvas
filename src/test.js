const { invoke } = window.__TAURI__.tauri;

let greetInputEl;
let greetMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}


class EventLogDate {
  year;
  month;
  day;
  hour;
  minute;
  second;
  millisecond;
}
class EventLog {
  event_id;
  content;
  time_generated;
}

async function debug() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  //setGreetMsg(await invoke("greet", { name }));
  //let msg = name;
  //setGreetMsg(await invoke("message_box", { msg }));
  let size = 100;
  let base = document.getElementById("log_body");
  //await invoke("message_box", {msg: "test"});
  const logs = await invoke("get_logon_logoff_log", { size: 100 });
  for (let i = 0; i < logs.length; i++) {
    let log = logs[i];
    let row = document.createElement("tr");
    // ID作成
    let tag_id = document.createElement("td");
    tag_id.textContent = log.event_id;
    // Message作成
    let tag_content = document.createElement("td");
    tag_content.textContent = log.content;
    // DateTime作成
    let tag_dt = document.createElement("td");
    let dt = new Date(log.time_generated.year, log.time_generated.month, log.time_generated.day, log.time_generated.hour, log.time_generated.minute, log.time_generated.second);
    tag_dt.textContent = dt.toLocaleString();
    //
    row.appendChild(tag_id);
    row.appendChild(tag_content);
    row.appendChild(tag_dt);
    //
    base.lastElementChild?.remove();
    base.appendChild(row);
  }

  if (logs.length === 0) {
    let row = document.createElement("tr");
    let tag_1 = document.createElement("td");
    tag_1.textContent = "Log Result 0."
    let tag_2 = document.createElement("td");
    let tag_3 = document.createElement("td");
    row.appendChild(tag_1);
    row.appendChild(tag_2);
    row.appendChild(tag_3);
    base.lastElementChild?.remove();
    base.appendChild(row);
  }
}




window.addEventListener("DOMContentLoaded", () => {
  //greetInputEl = document.querySelector("#greet-input");
  //greetMsgEl = document.querySelector("#greet-msg");
  document
    .querySelector("#debug-button")
    .addEventListener("click", () => debug());
});
