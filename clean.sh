#!/bin/sh

fd -t f . sql/ | while read -r sql_file; do
  if ! rg -F "$sql_file" --glob "!sql/*" . > /dev/null; then
    echo "Deleting unreferenced file: $sql_file"
    rm "$sql_file"
  else
    echo "Referenced: $sql_file"
  fi
done

echo "Cleanup complete."

