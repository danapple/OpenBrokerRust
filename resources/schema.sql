DROP TABLE IF EXISTS order_state_history;
DROP TABLE IF EXISTS trade;
DROP TABLE IF EXISTS order_state;
DROP TABLE IF EXISTS order_status;
DROP TABLE IF EXISTS order_leg;
DROP TABLE IF EXISTS order_base;
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

CREATE TABLE IF NOT EXISTS order_base (
      orderId BIGSERIAL PRIMARY KEY,
      accountKey VARCHAR NULL,
      extOrderId VARCHAR NOT NULL,
      clientOrderId VARCHAR NOT NULL,
      createTime BIGINT NOT NULL,
      price REAL NOT NULL,
      quantity INT NOT NULL
);

CREATE UNIQUE INDEX unq_accountKey_clientOrder ON order_base (accountKey, clientOrderId);
CREATE UNIQUE INDEX unq_accountKey_extOrder ON order_base (accountKey, extOrderId);

CREATE TABLE IF NOT EXISTS order_leg (
      orderLegId BIGSERIAL PRIMARY KEY,
      orderId BIGINT NOT NULL REFERENCES order_base,
      instrumentId BIGINT NOT NULL REFERENCES instrument,
      ratio BIGINT NOT NULL
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

GRANT SELECT, INSERT ON ALL TABLES IN SCHEMA public TO broker_user;
GRANT UPDATE ON TABLE public.order_state TO broker_user;

GRANT SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA public TO broker_user;