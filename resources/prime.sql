DELETE FROM CUSTOMER;
INSERT INTO CUSTOMER (customer_name) VALUES ('Daniel');
DELETE FROM API_KEY;
INSERT INTO API_KEY (customerId, apiKey) select customerId, 'DanApiKey' from CUSTOMER WHERE customer_name = 'Daniel';

DELETE FROM account;
INSERT INTO account (accountKey, accountNumber, accountName) VALUES ('retkey', '11112222', 'Retail');

DELETE FROM customer_account_relationship;
INSERT INTO customer_account_relationship (customerId, accountId, nickname) VALUES
((select customer.customerId from CUSTOMER WHERE customer_name = 'Daniel'),
(select account.accountId from ACCOUNT WHERE accountName = 'Retail' AND accountNumber = '11112222'),
'nick3');

INSERT INTO access (  relationshipId, privilege) VALUES (1, 'Read');
INSERT INTO access (  relationshipId, privilege) VALUES (1, 'Submit');

select * from customer_account_relationship;

insert into instrument (instrumentId, exchangeinstrumentid, exchangeurl, status) VALUES (0, 0, 'none', 'ACTIVE');
insert into instrument (instrumentId, exchangeinstrumentid, exchangeurl, status) VALUES (1, 1, 'none', 'ACTIVE');

-- DELETE FROM access;
-- INSERT INTO access (relationshipId, privilege)
