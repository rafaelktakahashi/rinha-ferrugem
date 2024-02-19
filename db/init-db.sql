-- THIS SCRIPT USES "char" THROUGHOUT INSTEAD OF CHAR BECAUSE
-- THE RUST LIBRARY sqlx MAPS i8 TO "char" AND THAT BEHAVIOR
-- CANNOT BE CHANGED
--
-- W IS WHEN, DEFAULTS TO START OF CURRENT TRANSACTION
CREATE TABLE T(
    U_ID "char" NOT NULL,
    VALOR INTEGER NOT NULL, -- IN CENTS
    TIPO BOOLEAN NOT NULL, -- TRUE FOR 'c', FALSE FOR 'd'
    DESCRICAO TEXT NOT NULL, -- IMPLICITLY ASCII ONLY, 1 TO 10 BYTES
    W TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- USERS
--
-- ID IS RECEIVED BY THE API AS 1, 2, 3, 4, 5, AND THEN CONVERTED
-- TO 'A', 'B', 'C', 'D', 'E' FOR STORAGE, FOR NO PARTICULAR REASON
CREATE TABLE U(
    ID "char" NOT NULL,
    LIMITE INTEGER NOT NULL,
    SALDO INTEGER NOT NULL -- OFTEN NEGATIVE
);

-- USERS, THE IMMORTALS
INSERT INTO U(ID, LIMITE, SALDO)
    VALUES('A', 100000, 0), -- ID 41
        ('B', 80000, 0), -- ID 42
        ('C', 1000000, 0), -- ID 43
        ('D', 10000000, 0), -- ID 44
        ('E', 500000, 0); -- ID 45

-- VERIFICATION, UPDATE AND INSERT ALL IN ONE ATOMICALLY
-- PRODUCES 1 ROW IF SUCCESSFUL, 0 ROWS IF DISALLOWED
CREATE OR REPLACE FUNCTION insert_into_t(u_id_arg "char", valor_arg INTEGER, tipo_arg BOOLEAN, descricao_arg TEXT)
RETURNS SETOF U AS $$ -- ALWAYS 0 OR 1 OF U
DECLARE
    user_record U%ROWTYPE;
BEGIN
    -- CHECK IF OPERATION IS ALLOWED
    -- 'c' IS ALWAYS PERMITTED, 'd' IS ONLY PERMITTED
    -- WHEN SALDO MINUS valor_arg WOULD NOT BECOME SMALLER THAN
    -- THE ADDITIVE INVERSE OF LIMIT
    IF tipo_arg OR (NOT tipo_arg AND EXISTS (
        SELECT 1 FROM U
        WHERE ID = u_id_arg
        AND SALDO - valor_arg >= -LIMITE
    )) THEN
        -- PERFORM THE INSERT INTO T
        -- IT IS ASSUMED THAT EACH VALUE HAS PREVIOUSLY BEEN
        -- VALIDATED AND THIS OPERATION WILL NOT FAIL
        INSERT INTO T (U_ID, VALOR, TIPO, DESCRICAO)
        VALUES (u_id_arg, valor_arg, tipo_arg, descricao_arg);
        
        -- UPDATE SALDO IN U AND GET THE UPDATED ROW
        UPDATE U
        SET SALDO = CASE WHEN tipo_arg THEN SALDO + valor_arg ELSE SALDO - valor_arg END
        WHERE ID = u_id_arg
        RETURNING * INTO user_record;
        
        -- RETURN THE UPDATED RECORD OF U
        RETURN NEXT user_record;
    ELSE
        -- RETURN NO ROWS IF THE OPERATION IS NOT PERMITTED
        RETURN;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- PARAMETERS
-- No clue what makes a difference
ALTER SYSTEM SET max_connections = 600;
ALTER SYSTEM SET shared_buffers = '0.2GB';
ALTER SYSTEM SET effective_cache_size = '100MB';
ALTER SYSTEM SET synchronous_commit = off;
ALTER SYSTEM SET work_mem = '6MB';
ALTER SYSTEM SET temp_buffers = '64MB';
ALTER SYSTEM SET fsync = off;
ALTER SYSTEM SET commit_delay = 40;
ALTER SYSTEM SET checkpoint_timeout = 86399;
ALTER SYSTEM SET log_statement = none;

-- DO NOT LOSE TIME ON DAILY TRIVIALITIES
-- DO NOT DWELL ON PETTY DETAIL
--
-- FOR ALL THESE THINGS MELT AWAY
-- AND DRIFT APART WITHIN THE OBSCURE TRAFFIC OF TIME
--
-- LIVE WELL AND LIVE BROADLY
-- YOU ARE ALIVE AND LIVING NOW
-- NOW IS THE ENVY OF ALL OF THE DEAD
--
-- (DON HERTZFELDT, WORLD OF TOMORROW)