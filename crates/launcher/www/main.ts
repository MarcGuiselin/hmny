import { listen } from "@tauri-apps/api/event";
import { appWindow } from "@tauri-apps/api/window";

type Handle = string;

type Status = {
  handle: Handle;
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

type TaskWithStatus = Task & {
  status: Status;
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
const tasks: Map<Handle, TaskWithStatus> = new Map();
const taskOverall = createTaskNode(true);

listen<Status>("loader_status_update", ({ payload: status }) => {
  // Get or create task
  let task = tasks.get(status.handle);
  if (!task) {
    task = {
      status,
      ...createTaskNode(),
    };
    tasks.set(status.handle, task);
  }

  // Update task progress
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

  // Update overall progress
  const tasksArray = Array.from(tasks.values());
  const tasksCount = tasksArray.length;
  taskOverall.$parent.style.visibility = tasksCount ? "visible" : "hidden";
  if (tasksCount) {
    const completedCount = tasksArray.filter(
      (task) => task.status.done_ratio == 1
    ).length;
    taskOverall.$title.textContent = `${completedCount} of ${tasksCount} Completed`;
    taskOverall.$done.style.width = `${(
      (completedCount / tasksCount) *
      100
    ).toFixed(2)}%`;
  }
});

window.addEventListener("DOMContentLoaded", () => {
  console.log("DOMContentLoaded");
  appWindow.show();
});
