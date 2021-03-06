HEADER = '\033[95m'
INFO = '\033[94m'
VERBOSE = '\033[96m'
SUCCESS = '\033[92m'
WARNING = '\033[93m'
ERROR = '\033[91m'
YELLOW = '\033[0;33m'
NO_COLOR = '\033[0m'
BOLD = '\033[1m'
UNDERLINE = '\033[4m'

IS_VERBOSE = False


def ask(msg: str):
    try:
        res = input(f"{INFO}{msg}[Enter] {NO_COLOR}")
    except KeyboardInterrupt:
        print("")
        return False

    return not res or res == "Y" or res == "y";


def log_success(log: str):
    print(f"{SUCCESS}{BOLD}{log}{NO_COLOR}")


def is_verbose_mode():
    return IS_VERBOSE


def set_global_verbose(v):
    global IS_VERBOSE
    IS_VERBOSE = v


def log_normal(log: str, bold=False):
    if bold:
        print(f"{BOLD}{log}{NO_COLOR}")
    else:
        print(log)


def log_verbose(log: str):
    if IS_VERBOSE:
        print(log)


def log_warning(log: str):
    print(f"{WARNING} {log}{NO_COLOR}")


def log_info(log: str, bold=False):
    if bold:
        print(f"{INFO}{BOLD} {log}{NO_COLOR}")
    else:
        print(f"{INFO} {log}{NO_COLOR}")


def log_error(log: str, exit: bool = False):
    print(f"{ERROR} {log}{NO_COLOR}")
    if exit:
        import sys
        sys.exit(1)
