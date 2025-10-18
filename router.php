<?php
$path = parse_url($_SERVER['REQUEST_URI'], PHP_URL_PATH);

if (strpos($path, '.js')) {
  header('Content-Type: text/javascript');
  readfile(__DIR__ . $path);
  exit;
}

return false;
?>
