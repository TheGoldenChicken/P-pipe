mkdir my-cron-job && cd my-cron-job
echo -e '#!/bin/bash\npython3 /app/dispatcher.py' > run.sh
chmod +x run.sh
