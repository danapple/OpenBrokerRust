INSERT INTO actor (actorName, emailAddress) VALUES ('admin', 'noemail@dev.null');

INSERT INTO admin_role (adminRoleName) VALUES ('All powers');

insert into admin_role_power (adminRoleId, power)
VALUES ((SELECT adminRoleId from admin_role where adminRoleName = 'All powers'), 'All');

insert into api_key (actorId, apiKey)
VALUES ((SELECT actorId from actor where emailAddress = 'noemail@dev.null'), 'InitialAdminApiKey');

insert into admin_role_membership (adminRoleId, actorId)
       VALUES
       ((SELECT actorId from actor where emailAddress = 'noemail@dev.null'), (SELECT adminRoleId from admin_role where adminRoleName = 'All powers'));


