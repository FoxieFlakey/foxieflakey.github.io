use std::sync::LazyLock;

use anyhow::Context;
use regex::Regex;
use rustyline::{
    Cmd, Completer, CompletionType, Config, Editor, EventHandler, Helper, Highlighter, Hinter,
    KeyCode, KeyEvent, Modifiers, Validator,
    completion::{FilenameCompleter, unescape},
    error::ReadlineError,
    history::MemHistory,
};
use tokio::sync::Mutex;

#[derive(Helper, Completer, Hinter, Validator, Highlighter)]
struct HelperThing {
    #[rustyline(Completer)]
    completer: FilenameCompleter,
}

static STDIN_EDITOR: LazyLock<Mutex<Editor<HelperThing, MemHistory>>> = LazyLock::new(|| {
    let cfg = Config::builder()
        .auto_add_history(true)
        .completion_type(CompletionType::List)
        .enable_signals(true)
        .check_cursor_position(true)
        .build();

    let mut editor =
        Editor::with_history(cfg, MemHistory::new()).expect("Cannot initialize rustyline editor");

    editor.set_helper(Some(HelperThing {
        completer: FilenameCompleter::new(),
    }));

    Mutex::new(editor)
});

async fn read_line<T: AsRef<str> + ?Sized>(
    prompt: &T,
    is_multiline: bool,
    is_file_path: bool,
    initial_value: Option<(&str, &str)>,
) -> Result<String, ReadlineError> {
    let mut editor = STDIN_EDITOR.lock().await;

    if is_multiline {
        editor.bind_sequence(
            KeyEvent(KeyCode::Enter, Modifiers::NONE),
            EventHandler::Simple(Cmd::Newline),
        );
        editor.bind_sequence(
            KeyEvent(KeyCode::Char('d'), Modifiers::CTRL),
            EventHandler::Simple(Cmd::AcceptLine),
        );
    } else {
        editor.unbind_sequence(KeyEvent(KeyCode::Enter, Modifiers::NONE));
        editor.unbind_sequence(KeyEvent(KeyCode::Char('d'), Modifiers::CTRL));
    }

    if is_file_path {
        #[cfg(unix)]
        const ESCAPE_CHAR: Option<char> = Some('\\');
        #[cfg(windows)]
        const ESCAPE_CHAR: Option<char> = None;

        Ok(unescape(
            &editor.readline_with_initial(prompt.as_ref(), initial_value.unwrap_or(("", "")))?,
            ESCAPE_CHAR,
        )
        .to_string())
    } else {
        Ok(editor.readline_with_initial(prompt.as_ref(), initial_value.unwrap_or(("", "")))?)
    }
}

pub async fn ask_question<T: AsRef<str> + ?Sized>(
    prompt: &T,
    is_multiline: bool,
    is_file_path: bool,
    default: Option<&str>,
    pattern: Option<(&Regex, &str)>,
    initial_value: Option<(&str, &str)>,
) -> anyhow::Result<Option<String>> {
    let prompt = prompt.as_ref();
    let default = default.map(AsRef::as_ref);

    loop {
        let prompt = match default {
            Some(x) => format!("{prompt} [{x}]: "),
            None => format!("{prompt}: "),
        };

        let response = read_line(&prompt, is_multiline, is_file_path, initial_value).await;
        if matches!(
            response,
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof)
        ) {
            // Canceled
            return Ok(None);
        }

        let response = response
            .context("Cannot read line from stdin line editor")?
            .trim()
            .to_string();

        if response.is_empty() {
            return Ok(default.map(str::to_string));
        }

        if let Some((pattern, error_msg)) = pattern {
            if pattern.is_match(&response) {
                return Ok(Some(response));
            }

            println!("Error, fix ur input :3 because {error_msg}");
        } else {
            return Ok(Some(response));
        }
    }
}
