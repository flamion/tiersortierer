ALTER TABLE animals
    ADD created_by BIGINT;


ALTER TABLE animals
    ADD FOREIGN KEY (created_by) REFERENCES users (user_id);
