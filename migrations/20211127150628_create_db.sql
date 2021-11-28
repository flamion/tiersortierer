--
-- Add migration script here
--

CREATE TABLE meta
(
    VERSION INT
);

CREATE TABLE species
(
    longhand  VARCHAR(256) UNIQUE PRIMARY KEY,
    shorthand VARCHAR(8) UNIQUE
);

CREATE TABLE vaccine
(
    id   SERIAL NOT NULL PRIMARY KEY,
    name TEXT UNIQUE
);

CREATE TABLE race
(
    id   BIGSERIAL PRIMARY KEY NOT NULL,
    name TEXT UNIQUE           NOT NULL
);

CREATE TABLE owner
(
    id                      BIGSERIAL NOT NULL PRIMARY KEY,
    title                   TEXT,
    last_name               TEXT      NOT NULL,
    first_name              TEXT      NOT NULL,
    zip_code                INT       NOT NULL,
    street_and_house_number TEXT      NOT NULL,
    telephone_number        TEXT,
    mobile_number           TEXT,
    email                   TEXT,
    member                  BOOLEAN   NOT NULL
);

CREATE TABLE animal_type
(
    longhand  TEXT PRIMARY KEY,
    shorthand TEXT UNIQUE
);

CREATE TABLE animals
(
    id                   BIGSERIAL NOT NULL PRIMARY KEY,
    custom_id            TEXT UNIQUE, --Handvergebene Buchnummer
    old_custom_id        TEXT UNIQUE, --Alte Buchnummer
    arrival_date         DATE      NOT NULL,
    species              VARCHAR   NOT NULL REFERENCES animal_type (shorthand) ON DELETE CASCADE,
    assigned_name        TEXT      NOT NULL,
    new_name             TEXT,
    is_male              BOOLEAN   NOT NULL,
    castration_date      DATE,
    weight               float8,
    race                 BIGINT REFERENCES race (id),
    description          TEXT      NOT NULL,
    date_of_birth        DATE,
    reason_for_surrender TEXT,
    old_owner            BIGINT REFERENCES owner (id),
    new_owner            BIGINT REFERENCES owner (id)
);

CREATE TABLE vaccinations
(
    vaccine                        INTEGER REFERENCES vaccine (id) ON DELETE CASCADE,
    animal                         BIGINT REFERENCES animals (id) ON DELETE CASCADE,
    date_of_vaccination            DATE,
    date_of_vaccination_expiration DATE
);
