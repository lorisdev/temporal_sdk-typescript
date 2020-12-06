import path from 'path';
import { Workflow } from './engine';
import * as stdlib from './stdlib';
export { Workflow };

async function run() {
  const example = path.join(__dirname, '../testScripts/lib/index.js');

  const workflow = await Workflow.create();
  await stdlib.install(workflow);
  await workflow.run(example);
  console.log('=== complete ===');

  // TODO: run in loop to test determinism
  // workflow.timeline.resetCursor();
  // const workflow2 = await Workflow.create(workflow.timeline);
  // await stdlib.install(workflow2);
  // await workflow2.run(example);
}

run().catch((err) => {
  console.error(err);
  process.exit(1);
});
