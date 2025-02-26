const concurrently = require('concurrently');

const {result} = concurrently([
  { command: 'cd packages/frontend && pnpm dev', name: 'frontend', prefixColor: 'blue' },
  { command: 'cargo lambda watch --bin api', name: 'backend', prefixColor: 'green' }
], {
  killOthers: ['failure', 'success'],
});

result
  .then(() => {
    console.log('Both processes have exited successfully.');
  })
  .catch((error) => {
    console.error('An error occurred while running the processes:', error);
  });
