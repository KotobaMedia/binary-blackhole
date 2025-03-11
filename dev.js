const concurrently = require('concurrently');
const { execSync } = require('child_process');

const DDB_CONT = 'bbh-ddb';

// Function to determine the appropriate DynamoDB command
function getDynamoDBCommand() {
  try {
    // Check if container is already running
    const runningContainers = execSync(`docker ps -q -f name=${DDB_CONT}`).toString().trim();

    if (runningContainers) {
      return `docker attach ${DDB_CONT}`;
    }

    // Check if container exists but is stopped
    const existingContainers = execSync(`docker ps -a -q -f name=${DDB_CONT}`).toString().trim();

    if (existingContainers) {
      return `docker start -a ${DDB_CONT}`;
    }

    // Container doesn't exist, need to create a new one
    return `docker run --name ${DDB_CONT} -p 9001:8000 amazon/dynamodb-local:latest -jar DynamoDBLocal.jar -sharedDb`;
  } catch (error) {
    console.error('Error checking Docker container status:', error);
    // If there's an error, return a command that will fail gracefully
    return 'echo "Failed to determine DynamoDB container status" && exit 1';
  }
}

const { result } = concurrently([
  { command: 'cd packages/frontend && pnpm dev', name: 'frontend', prefixColor: 'blue' },
  {
    command: 'cargo lambda watch --bin api -P 9000',
    name: 'backend',
    prefixColor: 'green',
    env: { RUST_BACKTRACE: '1' }
  },
  {
    command: 'cargo lambda watch --bin api-streaming --features="streaming" -P 8999',
    name: 'backend-streaming',
    prefixColor: 'green',
    env: { RUST_BACKTRACE: '1' }
  },
  { command: getDynamoDBCommand(), name: 'dynamodb', prefixColor: 'yellow' }
], {
  killOthers: ['failure', 'success'],
});

result
  .then(() => {
    console.log('All processes have exited successfully.');
  })
  .catch(() => {
    console.error('An error occurred while running the processes');
  });
