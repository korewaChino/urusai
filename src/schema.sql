-- SQLite Schema for the database

-- There are Discord servers, which also dictate configs for the bot.
CREATE TABLE IF NOT EXISTS servers (
    id INTEGER PRIMARY KEY,
    voice_channel INTEGER,
    text_channel INTEGER
);

CREATE TABLE IF NOT EXISTS users (
    id,
    server_id INTEGER,
    voice TEXT,

    FOREIGN KEY(server_id) REFERENCES servers(id)
);