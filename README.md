Haul is an advanced WIP todo list made in rust and sqlite

Commands :  
 - \[add task list\] adds a task to specified list  
 - \[list list\] lists the task in specified list  
 - \[remove -l list / -t id \] removes specified list with -l, removes specified task with -t  
 - \[done id\] marks specified task as done  
 - \[listall\] lists all the tasks and their respective lists present in the database  
 - \[reinstall\] resets the database  
 - \[configpath\] displays current config file path

version 0.3.0 :  
  Added a config file :  
    - Allows enabling/disabling colors and compact mode  
    - Allows to choose different date formats  
    - Allows to set the number of days before displaying a visual warning when the due date approaches (requires colors enabled)  
  Added the option to link a specific due date to your tasks  
  Added compact mode  
