# Version 1.1
- [x] Command to delete team
- [x] Daily double
- [x] Review TODO comments
- [x] Allow questions to have a custom duration
- [x] Create a quizmaster role instead of relying on admin privilege
- [x] Fixed a bug where trailing text after correct answers could be accepted
- [] Unit tests

## Version 1.0
- [x] Bot personality (image, name, texts)
- [x] Announce winning team
- [x] Preload songs onto disk
- [x] Quiz is only one z
- [x] Show all guesses after the answer
- [x] Bug where players can still votes with one category left
- [x] Fix instance of team name not being prefixed by Team and capitalized

## Beta
- [x] Fix bug where second guess loses full question value if first guess was also wrong
- [x] Sanitize team names
- [x] Post quiz updates to team channels (and allow voting there too, beware of dupe votes)
- [x] Reveal answer after all teams have submitted a guess
- [x] Remove vote for final question
- [x] Add sound effects for: question about to start, correct answer, wrong answer, time's up, channel join
- [x] Mention that players should guess game name, not song name
- [x] Mention that bot audio level can be adjusted
- [x] Shorten question duration by a few seconds

## Alpha
- [x] Warning for 30 and 10 seconds left
- [x] Remind of team commands at the start of setup phase
- [x] Remind of guess commands at the start of quiz phase
- [x] Command permissions
- [x] Command groups for quiz VS setup
- [x] Commands to pause/unpause
- [x] Command to skip question

## MVP
- [x] Time limit on question
- [x] Return to setup phase when quiz is over
- [x] Team channels
- [x] Guessing earns or removes points
- [x] Teams can only guess once
- [x] Multiple teams can get points
- [x] Voting
- [x] Categories
- [x] Regex/fuzzy guesses
- [x] Don't allow begin before joining voice room
- [x] Announce answers in chat after questions
- [x] Command to skip quiz phase
- [x] Recap score between questions
- [x] Join team mid-game if not already on a team
- [x] Go straight to results after last question
- [x] Command to adjust scores
- [x] Command to reset scores
- [x] Command to reset teams

## Backlog
- Support for &t in URLs
- Commands to tweak settings
- Command to end quiz
- Create game on ready and on join server instead of within get_game()
- Solo play
- Double Jeopardy rules
- Final Jeopardy rules
- Help command
- Fine-tune required permissions instead of giving the bot admin rights
- Disk persistence
- Don't rely on external executables of FFMPEG and Youtube-DL
