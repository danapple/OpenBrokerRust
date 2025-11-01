
DROP TABLE IF EXISTS balance;
DROP TABLE IF EXISTS position;
DROP TABLE IF EXISTS trade;

DROP TABLE IF EXISTS order_state_history;
DROP TABLE IF EXISTS order_state;
DROP TABLE IF EXISTS order_status;

DROP TABLE IF EXISTS order_leg;
DROP TABLE IF EXISTS order_base;
DROP TABLE IF EXISTS order_number_generator;

DROP TABLE IF EXISTS admin_role_power;
DROP TABLE IF EXISTS power;
DROP TABLE IF EXISTS admin_role_membership;
DROP TABLE IF EXISTS admin_role;

DROP TABLE IF EXISTS access;
DROP TABLE IF EXISTS privilege;
DROP INDEX IF EXISTS unq_relationship;
DROP TABLE IF EXISTS actor_account_relationship;
DROP TABLE IF EXISTS account;
DROP TABLE IF EXISTS api_key;
DROP TABLE IF EXISTS login_info;
DROP TABLE IF EXISTS actor;
DROP TABLE IF EXISTS offer;
DROP TABLE IF EXISTS instrument;
DROP TABLE IF EXISTS instrument_status;
DROP TABLE IF EXISTS asset_class;
DROP TABLE IF EXISTS exchange;
DROP TABLE IF EXISTS id;

CREATE TABLE IF NOT EXISTS id (
      idKey VARCHAR PRIMARY KEY,
      lastReservedId BIGINT
);

CREATE TABLE IF NOT EXISTS exchange (
    exchangeId SERIAL PRIMARY KEY,
    code VARCHAR UNIQUE NOT NULL,
    url VARCHAR NOT NULL,
    websocketUrl VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    apiKey VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS instrument_status (
    instrumentStatus VARCHAR PRIMARY KEY
);

INSERT INTO instrument_status (instrumentStatus) VALUES
      ('Active'),
      ('Inactive')
;

CREATE TABLE IF NOT EXISTS asset_class (
    assetClass VARCHAR PRIMARY KEY
);

INSERT INTO asset_class (assetClass)
VALUES
    ('Equity'),
    ('Option'),
    ('Commodity'),
    ('Future'),
    ('Forward'),
    ('Swap'),
    ('Bond'),
    ('Cryto')
;

CREATE TABLE IF NOT EXISTS instrument (
    instrumentId BIGSERIAL PRIMARY KEY,
    exchangeId INT REFERENCES exchange,
    exchangeInstrumentId BIGINT NOT NULL,
    status VARCHAR NOT NULL REFERENCES instrument_status,
    symbol VARCHAR NOT NULL,
    assetClass VARCHAR NOT NULL REFERENCES asset_class,
    description VARCHAR NOT NULL,
    expirationTime BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS offer (
    offerId SERIAL PRIMARY KEY,
    code VARCHAR UNIQUE NOT NULL,
    description VARCHAR NOT NULL,
    expirationTime BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS actor (
    actorId SERIAL PRIMARY KEY,
    actorName VARCHAR NOT NULL,
    emailAddress VARCHAR NOT NULL,
    offerId INT NULL REFERENCES offer
);

CREATE TABLE IF NOT EXISTS login_info (
    loginInfoId SERIAL PRIMARY KEY,
    actorId INT NOT NULL REFERENCES actor,
    passwordHash VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS api_key (
    apiKeyId SERIAL PRIMARY KEY,
    actorId INT NOT NULL REFERENCES actor,
    apiKey VARCHAR UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS account (
    accountId SERIAL PRIMARY KEY,
    accountKey VARCHAR NOT NULL,
    accountNumber VARCHAR UNIQUE NOT NULL,
    accountName VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS actor_account_relationship (
    relationshipId SERIAL PRIMARY KEY,
    actorId INT REFERENCES actor,
    accountId INT REFERENCES account,
    nickname VARCHAR NOT NULL
);

CREATE UNIQUE INDEX unq_relationship ON actor_account_relationship (actorId, accountId);

CREATE TABLE IF NOT EXISTS privilege (
        privilege VARCHAR PRIMARY KEY
);

INSERT INTO privilege (privilege) VALUES
    ('Owner'),
    ('Read'),
    ('Submit'),
    ('Cancel'),
    ('MakeMarkets'),
    ('Withdraw');

CREATE TABLE IF NOT EXISTS access (
    accessId SERIAL PRIMARY KEY,
    relationshipId INT NOT NULL REFERENCES actor_account_relationship,
    privilege VARCHAR NOT NULL REFERENCES privilege
);

CREATE UNIQUE INDEX unq_priv ON access (relationshipId, privilege);

CREATE TABLE IF NOT EXISTS admin_role (
    adminRoleId SERIAL PRIMARY KEY,
    adminRoleName VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS admin_role_membership (
    adminRoleMembershipId SERIAL PRIMARY KEY,
    adminRoleId INT REFERENCES admin_role,
    actorId INT REFERENCES actor
);

CREATE UNIQUE INDEX unq_admin_role_membership ON admin_role_membership (actorId, adminRoleId);

CREATE TABLE IF NOT EXISTS power (
    power VARCHAR PRIMARY KEY
);

INSERT INTO power (power)
VALUES
    ('All'),
    ('Read');

CREATE TABLE IF NOT EXISTS admin_role_power (
    adminRolePowerId SERIAL PRIMARY KEY,
    adminRoleId INT NOT NULL REFERENCES admin_role,
    power VARCHAR NOT NULL REFERENCES power
);

CREATE UNIQUE INDEX unq_admin_role_power ON admin_role_power (adminRoleId, power);


CREATE TABLE IF NOT EXISTS order_number_generator (
    accountId INT PRIMARY KEY REFERENCES account,
    lastOrderNumber INT
);

CREATE TABLE IF NOT EXISTS order_base (
      orderId BIGSERIAL PRIMARY KEY,
      accountId INT NOT NULL REFERENCES account,
      orderNumber INT NOT NULL,
      extOrderId VARCHAR NOT NULL,
      clientOrderId VARCHAR NOT NULL,
      createTime BIGINT NOT NULL,
      price REAL NOT NULL,
      quantity INT NOT NULL
);

CREATE UNIQUE INDEX unq_accountId_clientOrder ON order_base (accountId, clientOrderId);
CREATE UNIQUE INDEX unq_accountId_extOrder ON order_base (accountId, extOrderId);
CREATE UNIQUE INDEX unq_accountId_orderNumber ON order_base (accountId, orderNumber);

CREATE TABLE IF NOT EXISTS order_leg (
      orderLegId BIGSERIAL PRIMARY KEY,
      orderId BIGINT NOT NULL REFERENCES order_base,
      instrumentId BIGINT NOT NULL REFERENCES instrument,
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

CREATE TABLE IF NOT EXISTS position (
    positionId BIGSERIAL PRIMARY KEY,
    accountId INT NOT NULL REFERENCES account,
    instrumentId BIGINT NOT NULL REFERENCES instrument,
    cost REAL NOT NULL,
    quantity INT NOT NULL,
    closedGain REAL NOT NULL,
    updateTime BIGINT NOT NULL,
    versionNumber BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS balance (
    balanceId SERIAL PRIMARY KEY,
    accountId INT UNIQUE NOT NULL REFERENCES account,
    cash REAL NOT NULL,
    updateTime BIGINT NOT NULL,
    versionNumber BIGINT NOT NULL
);

GRANT SELECT ON TABLE privilege, power TO broker_user;

GRANT SELECT, INSERT ON TABLE order_base, order_number_generator, order_leg, order_status, order_state,
    order_state_history, trade, position, balance, actor, login_info, offer, account, balance,
    actor_account_relationship, access, api_key, exchange, instrument
    TO broker_user;

GRANT UPDATE ON TABLE public.order_state, public.position, public.balance, public.order_number_generator,
    public.login_info
    TO broker_user;

GRANT SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA public TO broker_user;