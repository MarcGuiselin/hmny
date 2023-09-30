use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard},
};
use tauri::Manager;
use tokio::sync::mpsc;

use crate::task;

const MAX_CONCURRENT_TASKS: usize = 3;

pub struct State(Arc<Mutex<Inner>>);

impl Clone for State {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

struct Inner {
    task_queue: VecDeque<Option<task::Task>>,
    active_tasks: Vec<task::ActiveTask>,
    send_task_update: task::StatusSender,
    receive_task_update: Option<task::StatusReceiver>,
}

impl Inner {
    fn is_initiated(&self) -> bool {
        self.receive_task_update.is_none()
    }
}

impl State {
    pub fn new() -> Self {
        let (send_task_update, receive_task_update) = mpsc::channel(1);
        // let (send_frontend, receive_frontend) = mpsc::channel(1);

        let inner = Inner {
            task_queue: VecDeque::new(),
            active_tasks: Vec::new(),
            send_task_update,
            receive_task_update: Some(receive_task_update),
        };

        Self(Arc::new(Mutex::new(inner)))
    }

    pub fn initiate<R: tauri::Runtime>(&self, handle: tauri::AppHandle<R>) {
        let mut inner = self
            .0
            .lock()
            .expect("failed to lock() in State::initiate 1");
        let mut receive_task_update = inner
            .receive_task_update
            .take()
            .expect("state initiated twice");

        // Perform actions on task status updates
        let inner_clone = Arc::clone(&self.0);
        tauri::async_runtime::spawn(async move {
            while let Some(status) = receive_task_update.recv().await {
                if status.is_completed() {
                    let mut inner = inner_clone
                        .lock()
                        .expect("failed to lock() in State::initiate 2");

                    // Remove completed task from the active list
                    inner
                        .active_tasks
                        .retain(|active_task| active_task.handle != status.handle);

                    // Process the queue
                    process_queue(inner);
                }

                // Notify the frontend of the task status
                handle.emit_all("loader_status_update", status).unwrap();
            }
        });

        // Initiate for the first time
        process_queue(inner);
    }

    pub fn enqueue_task(&self, task: task::Task) {
        let mut inner = self
            .0
            .lock()
            .expect("failed to lock() in State::enqueue_task");

        let task = Some(task);

        // Replace any duplicates on the queue with a None (no need for expensive reordering)
        inner
            .task_queue
            .iter_mut()
            .filter(|queued_task| **queued_task == task)
            .for_each(|queued_task| *queued_task = None);

        inner.task_queue.push_back(task);

        process_queue(inner);
    }
}

fn process_queue(mut inner: MutexGuard<'_, Inner>) {
    if !inner.is_initiated() {
        while inner.active_tasks.len() < MAX_CONCURRENT_TASKS {
            match inner.task_queue.pop_front() {
                None => break,
                Some(None) => {}
                Some(Some(task)) => {
                    let send_task_update = inner.send_task_update.clone();
                    inner.active_tasks.push(task.into_active(send_task_update));
                }
            }
        }
    }
}
