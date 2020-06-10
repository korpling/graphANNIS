CREATE TABLE corpus_groups (
    "group" VARCHAR NOT NULL,
    corpus VARCHAR NOT NULL,
    PRIMARY KEY("group", corpus)
);