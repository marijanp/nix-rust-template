let
  marijan.keys = [ "age1yubikey1q0tpa48d03dy59jcsjsx5a8zv0p8msr89ut7xgr64x5ujgkrn0ceulx4zwv" ];
  system.keys = [ ];
in
{
  "openid-client-id.age".publicKeys = marijan.keys ++ system.keys;
  "openid-client-secret.age".publicKeys = marijan.keys ++ system.keys;
}
