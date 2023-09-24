import { listen } from "@tauri-apps/api/event";

type TaskStatus = {
  title: string;
  status: string | null;
  done_ratio: number;
  doing_ratio: number;
  error: string | null;
};

type Task = {
  $parent: HTMLElement;
  $title: HTMLSpanElement;
  $status: HTMLDivElement;
  $loader?: HTMLSpanElement;
  $doing: HTMLDivElement;
  $done: HTMLDivElement;
};

const createTaskNode = (overall = false): Task => {
  const $title = document.createElement("span");

  const $left = document.createElement("div");
  $left.classList.add("left");
  $left.appendChild($title);

  const $status = document.createElement("div");
  $status.classList.add("right");

  const $clear = document.createElement("div");
  $clear.classList.add("clear");

  const $doing = document.createElement("div");
  $doing.classList.add("doing");

  const $done = document.createElement("div");
  $done.classList.add("done");
  $done.classList.add("swoosh-loader");

  const $loadingBar = document.createElement("div");
  $loadingBar.classList.add("loading-bar");
  $loadingBar.appendChild($doing);
  $loadingBar.appendChild($done);

  const $parent = document.createElement(overall ? "div" : "li");
  $parent.appendChild($left);
  $parent.appendChild($status);
  $parent.appendChild($clear);
  $parent.appendChild($loadingBar);

  let $loader: HTMLSpanElement | undefined;
  if (overall) {
    $parent.id = "tasks-overall";
    $taskMenu.insertBefore($parent, $tasks);
  } else {
    $loader = document.createElement("span");
    $loader.classList.add("loader");
    $loader.appendChild(document.createElement("span"));
    $loader.appendChild(document.createElement("span"));
    $loader.appendChild(document.createElement("span"));
    $left.appendChild($loader);

    $tasks.appendChild($parent);
  }

  return { $parent, $title, $status, $loader, $doing, $done };
};

const $taskMenu = document.getElementById("task-menu") as HTMLDivElement;
const $tasks = document.getElementById("tasks") as HTMLUListElement;
const tasks: Task[] = [];
const taskOverall = createTaskNode(true);

listen<TaskStatus[]>("loader_status_update", ({ payload: statuses }) => {
  console.log("loader_status_update", statuses);

  // Creates any missing tasks
  for (let i = tasks.length; i < statuses.length; i++) {
    tasks.push(createTaskNode());
  }

  // Delete any extra tasks
  while (tasks.length > statuses.length) {
    const task = tasks.pop();
    if (task) {
      $tasks.removeChild(task.$parent);
    }
  }

  // Update task progress
  for (let i = 0; i < statuses.length; i++) {
    const task = tasks[i];
    const status = statuses[i];
    const completed = status.done_ratio == 1;

    task.$title.textContent = status.title;
    task.$status.textContent = status.error || status.status || "";
    const width = `${(status.done_ratio * 100).toFixed(2)}%`;
    task.$done.style.width = width;
    task.$doing.style.left = width;
    task.$doing.style.right = `${(status.doing_ratio * 100).toFixed(2)}%`;
    if (task.$loader) {
      task.$loader.style.opacity = completed ? "0" : "1";
    }
  }

  // Update overall progress
  const completed = statuses.filter((status) => status.done_ratio === 1).length;
  taskOverall.$title.textContent = `${completed} of ${statuses.length} Completed`;
  taskOverall.$done.style.width = `${(
    (completed / statuses.length) *
    100
  ).toFixed(2)}%`;
});

window.addEventListener("DOMContentLoaded", () => {
  console.log("DOMContentLoaded");
});
