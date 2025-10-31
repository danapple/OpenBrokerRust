INSERT INTO actor (actorName, emailAddress) VALUES ('admin', 'noemail@dev.null');

INSERT INTO admin_role (adminRoleName) VALUES ('All powers');

insert into admin_role_power (adminRoleId, power)
VALUES ((SELECT adminRoleId from admin_role where adminRoleName = 'All powers'), 'All');

insert into api_key (actorId, apiKey)
VALUES ((SELECT actorId from actor where emailAddress = 'noemail@dev.null'), 'InitialAdminApiKey');

insert into admin_role_membership (adminRoleId, actorId)
       VALUES
       ((SELECT actorId from actor where emailAddress = 'noemail@dev.null'), (SELECT adminRoleId from admin_role where adminRoleName = 'All powers'));




-- DELETE FROM API_KEY;
-- INSERT INTO API_KEY (actorId, apiKey) select actorId, 'DanApiKey' from ACTOR WHERE actor_name = 'Daniel';
--
-- DELETE FROM account;
-- INSERT INTO account (accountKey, accountNumber, accountName) VALUES ('retkey', '11112222', 'Retail');
--
-- DELETE FROM actor_account_relationship;
-- INSERT INTO actor_account_relationship (actorId, accountId, nickname) VALUES
-- ((select actor.actorId from ACTOR WHERE actor_name = 'Daniel'),
-- (select account.accountId from ACCOUNT WHERE accountName = 'Retail' AND accountNumber = '11112222'),
-- 'nick3');
--
-- INSERT INTO access (  relationshipId, privilege) VALUES (1, 'Read');
-- INSERT INTO access (  relationshipId, privilege) VALUES (1, 'Submit');
--
-- select * from actor_account_relationship;
--
-- insert into instrument (instrumentId, exchangeinstrumentid, exchangeurl, status) VALUES (0, 0, 'none', 'ACTIVE');
-- insert into instrument (instrumentId, exchangeinstrumentid, exchangeurl, status) VALUES (1, 1, 'none', 'ACTIVE');

-- DELETE FROM access;
-- INSERT INTO access (relationshipId, privilege)
