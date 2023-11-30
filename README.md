### Server Machine
- Should have a local port open, prepared to accept a new task
- Should provide an easy entry point to working with the task
- Should have a common service for
	- listing available services, and the structure for calling them
	- subscribing to habit outputs
	- unsubscribing from habit outputs
	- ssh-ing into the workspace

### Remote Machines
- Should have a "start new remote task" command
- Should be able to specify all the fields of the task in a task file. This will be maintained by the user, and parsed by remote

### thing.task prototype
```toml
task_name = "" # (must be unique)
task_type = "" # (daemon)|(habit)|(service)
			   # daemon  -> it will run the specified command as a background process. e.g a game server.
			   # habit   -> it will run the specified command on an interval. e.g checking the price of an item online
			   # service -> it will run the specified command and return a result. e.g accepting an image file, and running an image recognition model on the script
commands = []  
setup_commands = []
necessary_files = []
run_in_docker = false


[task_details]      # not specific, applies to any task
on_end     = "none" # (none)(email)|(qmail)|(...) TODO think of more ways to notify for failure
on_fail    = "none" # see on_end
on_success = "none" # see on_end

[daemon_details]
restart_on_end = false
restart_on_fail = false
restart_on_success = false # successful commands in a daemon are sort of undefined behavior. this will be up to implementation
wait_after_setup = false   # in case you want to ssh in and perform some steps. there should be a system for modifying the task in this process...

[habit_details]
time_between_runs = ""       # a measurement of time for the habit to recur
ignore_nonzero_exits = false # ignore when the command fails

[service_details]
service_fail_is_command_fail = false # to differentiate command failure, and result failure. 

[docker_settings] # if run_in_docker is false, we ignore all docker_settings
docker_img_id = "Ubuntu 22.04"
start_container_on_reboot = false
```
### Examples
##### 1. habit
- **\$C** = client, **\$H** = host
- **\$C** has a binary/script called "my_binary"
- "my_binary" has an argument "--timeout=10", which indicates it will attempt some set of instructions until it either succeeds or 10 seconds have passed
- **\$C** wants to use the command every 5 minutes, 
- **\$C** uses a command, which generates the following task file:
```toml
[general]
task_name = "my_binary"
task_type = "habit"
commands = ["./my_binary --timeout=10"]
necessary_files = ["/home/matthew/scripts/my_binary"]
run_in_docker = true

[task_details] # not specific, applies to any task
on_success = "email"

[habit_details]
time_between_runs = "5m"
ignore_nonzero_exits = false

[docker_details]
docker_img_id = "Ubuntu 22.04"
apt_pkgs = []
pip_pkgs = []
start_container_on_reboot = false
```
-  one that finishes, it gets sent over a socket server or zenoh to **\$H**. **\$H** accepts the toml contents and the necessary files.
	- **\$H** stores the necessary files in a directory, downloads an Ubuntu 22.04 image, and shares the directory with the newly running container.
	- **\$H** launches the container with the command, and must monitor the status of the command
	- when **\$H** detects the command succeeds, it sends an email with a specified format

##### 2. daemon
- **\$C** = client, **\$H** = host
- **\$C** wants to host a game server using the daemon task
- **\$C** will submit the script for generating server files.
- The generated start script has low default ram allocation, so 
	- **\$C** will need **\$H** to run the setup commands
	- **\$C** will then ssh into **\$H** and modify the script
	- **\$C** will exit ssh, and submit a modification to the task
```toml
task_name = "Game Server" # (must be unique)
task_type = "daemon"
commands = []
setup_commands = ["chmod +x ./game-server-installer", "./game-server-installer"]
# note: if "game-server-installer" required user input, we're better off building the server locally and passing the entire directory to the host
# however, there should be a method for creating a task that does nothing, ssh-ing in to run the installer, and then modifying the task to run the start script
necessary_files = ["./game-server-installer"]
run_in_docker = true

[task_details]
on_end     = "email"
on_fail    = "email"
on_success = "qmail" # less important, so don't send an email, but I might as well get notified that I have a task with undefined behavior. lol.

[daemon_details]
restart_on_end = true
restart_on_fail = true
restart_on_success = true
wait_after_setup = true

[docker_settings]
docker_img_id = "Ubuntu 22.04"
apt_packages = ["openjdk8-jdk"] # setup commands will be run AFTER the apt packages are installed...
start_container_on_reboot = false
```
- Based on the above, the following occurs:
	- A task is generated, with a folder matching the task name in our `~/.tasks` directory
	- `game-server-installer` is remote-copied to **\$H** into the `~/.tasks/Game_Server` directory (dir name based on task name)
	- An Ubuntu 22.04 docker image is downloaded, and 'openjdk8-jdk' is downloaded onto it.
	- A container is started from the aforementioned image, with the task directory mounted to the container
	- The setup commands are executed, and then the container exits
	- Because the "wait_after_setup" flag is set to true, we notify **\$C** that the setup process has completed. **\$C** will ssh in, modify the start script, and exit
	- **\$C** will then send an edit-task message, informing **\$H** that the task has a new command and deciding whether it should re-run the setup commands. In this case, **\$C** does not want to re-run the setup commands.
	- Now, the docker container starts again and performs the necessary commands. All complete.

### Client Formatting
```
$ tnet --generate-only --type=service --run="myscript.py" --files=./*
# -> Creates 'servicetask.toml' of type "service" with default entries. Service executes
#    myscript.py on service calls, and attaches all files in current dir

$ tnet --generate-only --name=temperature_monitor --type=habit --edit
# -> Creates 'habittask.toml' of type habit with a name of "temperature_monitor" and
#    otherwise default values. Uses default editor after generating boilerplate.

$ tnet --generate-only --menu
Which type of task do you want?
->  Daemon  (Task runs as a background process, like a web server)
    Habit   (Task runs every X interval, like a system backup)
	  Service (Task runs when requested, like a printer request)

Daemon selected.

Choose a detail to edit (or 'q' to exit):
    Task Type (*unsaved)
->	Task Name
		Service Settings
		Docker Settings
    Write Changes

exiting...
```

