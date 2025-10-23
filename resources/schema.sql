DROP TABLE IF EXISTS order_state_history;
DROP TABLE IF EXISTS trade;
DROP TABLE IF EXISTS order_state;
DROP TABLE IF EXISTS order_status;
DROP TABLE IF EXISTS order_leg;
DROP TABLE IF EXISTS order_base;

DROP TABLE IF EXISTS access;
DROP TABLE IF EXISTS privilege;
DROP TABLE IF EXISTS customer_account_relationship;
DROP TABLE IF EXISTS account;
DROP TABLE IF EXISTS api_key;
DROP TABLE IF EXISTS customer;

DROP TABLE IF EXISTS instrument;
DROP TABLE IF EXISTS id;

CREATE TABLE IF NOT EXISTS id (
      idKey VARCHAR PRIMARY KEY,
      lastReservedId BIGINT
);

CREATE TABLE IF NOT EXISTS instrument (
      instrumentId BIGINT PRIMARY KEY,
      exchangeInstrumentId BIGINT NOT NULL,
      exchangeUrl VARCHAR NOT NULL,
      status VARCHAR NOT NULL
);

CREATE UNIQUE INDEX unq_exchange_instrument ON instrument (exchangeUrl, exchangeInstrumentId);

CREATE TABLE IF NOT EXISTS customer (
    customerId BIGSERIAL PRIMARY KEY,
    customerName VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS api_key (
    apiKeyId BIGSERIAL PRIMARY KEY,
    customerId BIGINT NOT NULL REFERENCES customer,
    apiKey VARCHAR UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS account (
    accountId BIGSERIAL PRIMARY KEY,
    accountKey VARCHAR NOT NULL,
    accountNumber VARCHAR UNIQUE NOT NULL,
    accountName VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS customer_account_relationship (
    relationshipId BIGSERIAL PRIMARY KEY,
    customerId BIGINT REFERENCES customer,
    accountId BIGINT REFERENCES account,
    nickname VARCHAR NOT NULL
);

CREATE UNIQUE INDEX unq_relationship ON customer_account_relationship (customerId, accountId);

CREATE TABLE IF NOT EXISTS privilege (
        privilege VARCHAR PRIMARY KEY
);

INSERT INTO privilege (privilege) VALUES
    ('Owner'),
    ('Read'),
    ('Submit'),
    ('Cancel');

CREATE TABLE IF NOT EXISTS access (
    accessId BIGSERIAL PRIMARY KEY,
    relationshipId BIGINT NOT NULL REFERENCES customer_account_relationship,
    privilege VARCHAR NOT NULL REFERENCES privilege
);

CREATE UNIQUE INDEX unq_priv ON access (relationshipId, privilege);

CREATE TABLE IF NOT EXISTS order_base (
      orderId BIGSERIAL PRIMARY KEY,
      accountId BIGINT NOT NULL REFERENCES account,
      extOrderId VARCHAR NOT NULL,
      clientOrderId VARCHAR NOT NULL,
      createTime BIGINT NOT NULL,
      price REAL NOT NULL,
      quantity INT NOT NULL
);

CREATE UNIQUE INDEX unq_accountId_clientOrder ON order_base (accountId, clientOrderId);
CREATE UNIQUE INDEX unq_accountId_extOrder ON order_base (accountId, extOrderId);

CREATE TABLE IF NOT EXISTS order_leg (
      orderLegId BIGSERIAL PRIMARY KEY,
      orderId BIGINT NOT NULL REFERENCES order_base,
      instrumentId BIGINT NOT NULL, --  REFERENCES instrument,
      ratio INT NOT NULL
);

CREATE TABLE IF NOT EXISTS order_status (
      orderStatus VARCHAR PRIMARY KEY
);

INSERT INTO order_status (orderStatus) VALUES
      ('Rejected'),
      ('Pending'),
      ('Open'),
      ('Filled'),
      ('PendingCancel'),
      ('Canceled'),
      ('Expired') ;

CREATE TABLE IF NOT EXISTS order_state (
      orderId BIGINT PRIMARY KEY REFERENCES order_base,
      orderStatus VARCHAR NOT NULL REFERENCES order_status,
      updateTime BIGINT NOT NULL,
      versionNumber BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS order_state_history (
      orderStateHistoryId BIGSERIAL PRIMARY KEY,
      orderId BIGINT REFERENCES order_base NOT NULL,
      orderStatus VARCHAR NOT NULL REFERENCES order_status,
      createTime BIGINT NOT NULL,
      versionNumber BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS trade (
      tradeId BIGSERIAL PRIMARY KEY,
      orderLegId BIGINT NOT NULL REFERENCES order_leg,
      createTime BIGINT NOT NULL,
      price FLOAT NOT NULL
);


GRANT SELECT ON TABLE customer, api_key, account, customer_account_relationship, privilege,access TO broker_user;

GRANT SELECT, INSERT ON TABLE order_base, order_leg, order_status, order_state, order_state_history, trade TO broker_user;
GRANT UPDATE ON TABLE public.order_state TO broker_user;

GRANT SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA public TO broker_user;