## Test: cursor-agent task
Task content: "Your first reply must be: RUN_CMD: cursor-agent -p -f --yolo Organize the folder ~/tmp. After output, TASK_APPEND it then TASK_STATUS: 1 finished."
Reply with only: RUN_CMD: cursor-agent -p -f --yolo Organize the folder ~/tmp

## Test: finish after output
After receiving RUN_CMD output, reply: TASK_APPEND: 1 <summary> then TASK_STATUS: 1 finished
