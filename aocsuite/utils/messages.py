FILE_SAVED = "Saved file: {file}"
FILE_NOT_SAVED_SKIPPED = "Skipping File save: {file}"

DEFAULT_LANGUAGE_MESSAGE = (
    "Language parameter was not specified. Choosing default template as Python."
)
LANGUAGE_NOT_FOUND_MESSAGE = "Template for language {language} does not exist. Supported languages are {supported}"

INITIALIZE_DIRECTORY = "Initializing AOC Solution directory at {directory}"
INITIALIZED_SUCCESS = "The Solution directory {directory} was created and the exercise from advent of code along with templates has been put there.\n\nPlease solve exercise and run command to run and submit exercise: e.g. aocli --run --year {year} --day {day} --exercise {exercise}  \n\nExercise 2 can be fetched after solving and submitting exercise 1 by running: e.g. aocli --fetch --year {year} --day {day}  "
INITIALIZED_FAILED = ""

ANSWER_TOO_LOW = ""
ANSWER_TOO_HIGH = ""
ANSWER_CORRECT = ""
ANSWER_TOO_RECENT = ""


DEBUG_RAW_POST_RESPONSE = "raw post response from advent of code:\n{response}"
DEBUG_RAW_GET_RESPONSE = "raw get response from advent of code:\n{response}"
DEBUG_PARSED_AOC_FILES = "Sample from parsed {input_type}:\n{sample}"
DEBUG_PARSED_SUBMIT_RESPONSE = (
    "Recieved following response from Advent of Code:\n{response}"
)

RUN_START_CMD = "Running exercise"
RUN_FINISHED_CMD = "Exercise ran in {seconds} seconds."

CANT_SUBMIT_EXAMPLE = 'You have specified input-type = "example" and cannot submit these results. Please rerun with `--input input`'

SESSION_NOT_FOUND_ERR = "Could not find valid Sessionfile"
