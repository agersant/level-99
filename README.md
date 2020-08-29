[![Actions Status](https://github.com/agersant/level-99/workflows/Build/badge.svg)](https://github.com/agersant/level-99/actions)

# Level-99

Level-99 is a self-hosted bot which can run blind-tests on your Discord server. Teams of players compete for points, trying to recognize music being played by the bot.

<img src="res/readme/demo.png?raw=true"/>

# Setup Instructions

## Dependencies

- Download an executable of [youtube-dl](https://ytdl-org.github.io/youtube-dl/download.html) for your system. Make sure to add it to your path and that it can be invoked by executing `youtube-dl` in a console window.
- Download an executable of [FFmpeg](https://ffmpeg.org/download.html) for your system. Make sure to add it to your path and that it can be invoked by executing `youtube-dl` in a console window.
- Install the [Rust toolchain](https://rustup.rs/).

## Level-99 executable

- Clone this repository to your computer
- From the top-level directory of the repository, execute `cargo build --release`. This will create a `level99` executable within `target/release` which you can leave there, or move somewhere convenient on your system. Run this program whenever you want to use the bot.

## Create and Invite the Bot

- Follow [these instructions](https://discordpy.readthedocs.io/en/latest/discord.html)
- Make sure you list at least the following in required permissions:
    - Manage Roles
	- Manage Channels
	- View Channels
	- Send Messages
	- Add Reactions
	- Connect
	- Speak
- Feel free to customize your bot's name and icon
- From the `Bot` tab on the Discord website, click the `Copy` button to copy your bot's Token. Save it on your computer as an environment variable named `DISCORD_TOKEN_LEVEL99`. Please note that this Token is different from the `CLIENT ID` and `CLIENT SECRET` on the `General Information` page for your Discord App.

# Preparing a Quiz

Quiz are CSV files listing questions, answers, point values, categories and other details about how your quiz should go. A wide variety of programs can be used to author these CSV files. Some which have been proved to work well are [Notion](https://www.notion.so/) and [LibreOffice Calc](https://www.libreoffice.org/discover/calc/). Regardless of which program you use, it is recommended that you start off using the `ExampleQuiz.csv` present in this repository.

### Understanding the various columns

- `url`: This column must contain a Youtube URL to the video whose audio will play when the question is asked.
- `answer`: This column must contain the answer which the bot will display as the expected answer at the end of the question.
- `category`: This column must contain the category associated with the question. During the quiz, players can vote for which category they want the next question to be from. Within a category, questions are always asked in ascending score value.
- `score_value`: This column must contain the number of points awarded for answering this question first.
- `acceptable_answers`: This column can be blank. It is used to list alternative answers which acceptable, in addition to the one in the `answer` column. Multiple entries can be separated using the `|` character. **Note that accents, capitalization and whitespace are all ignored - which means you don't need to list out these trivial variations**.
- `challenge`: This column can be blank. If it contains the word `TRUE`, the question will be a Challenge Question. These questions can only be answered by the team who last answered correctly, and the team will have the ability to wager a variable amount of points before the question begins.
- `duration_seconds`: This column can be blank. By default, each question lasts approximately 90 seconds. If a number is present in this column, it will the question's duration.

### Common authoring problems

- Make sure the first line of your CSV file contains column names.
- Make sure your CSV file is using UTF-8 text encoding.
- Sometimes videos are removed from Youtube, or are not available from the country you are running the bot from. When that happens, the bot will send an error message ("The quiz contains some songs that could not be downloaded") to Discord, and its console output will list the Youtube ID of the offending video.

# Running a Quiz

## Starting the quiz

- Make sure you followed all the instructions above and invited the bot to your server
- Gather all your friends in a voice channel
- Use the `!join` command for the bot to enter your voice channel
- Let your friends organize themselves into teams by using the `!team some-cool-name` command. Each team gets its own text channel to play the game in.
- When you are ready to start the quiz use the `!begin path-to-quiz-file.csv` command to start the quiz. The path can be relative to the directory you are running the bot from (eg. `ExampleQuiz.csv`), or absolute (eg. `C:\Level99\ExampleQuiz.csv`).
- Wait a bit while players are reading the rules and the bot is downloading all the audio that will be playing during the quiz.
- Players can use the `!guess` (and sometimes `!wager`) commands to play the game, as explained by the bot.

## Moderating the quiz

Server administrators or members with the `quizmaster` role can use commands to control the flow of the game:

- `!pause` and `!unpause` can be used to do breaks. Note pausing while a question is playing does not interrupt the audio, but it does interrupt the counting of time. It is recommended to do pauses during category votes.
- `!score team-name delta` can be used to add or remove points from a team. For example `!score kupo -400` would remove 400 points from team kupo.
- `!skip` can be used to advance between quiz phases (vote, question, cooldown) without delay.
- `!disband team-name` can be used to delete a team.
- `!end` can be used to stop the quiz entirely.
- `!reset scores` can be used to set all team scores to 0.
- `!reset teams` can be used to dissolve all teams.
