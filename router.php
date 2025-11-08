<?php
# Its doesnt contain HTTP scheme or anything just bare paths
$path = strtok(strtok($_SERVER['REQUEST_URI'], '#'), '?');
$documentRoot = $_SERVER['DOCUMENT_ROOT'];

if (
  (!is_dir($documentRoot . $path) && !file_exists($documentRoot . $path)) ||
  (is_dir($documentRoot . $path) && !file_exists($documentRoot . $path . "/index.html"))
) {
  http_response_code(404);
  readfile($documentRoot . "/404.html");
  exit;
}

if (strpos($path, '.js')) {
  header('Content-Type: text/javascript');
  readfile($documentRoot . $path);
  exit;
}

return false;
?>
