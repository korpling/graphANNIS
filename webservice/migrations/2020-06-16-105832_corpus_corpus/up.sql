CREATE TABLE groups (
    "name" VARCHAR NOT NULL,
	PRIMARY KEY("name")
);

CREATE TABLE corpus_groups (
    "group" VARCHAR NOT NULL REFERENCES groups("name") ON DELETE CASCADE ON UPDATE CASCADE,
    corpus VARCHAR NOT NULL,
    PRIMARY KEY("group", corpus)
);