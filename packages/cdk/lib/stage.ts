import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import { Stage } from 'aws-cdk-lib';
import { MainStack } from './main-stack';

export function getStageName(scope: Construct): string {
  const stage = Stage.of(scope);
  const stageName = stage?.stageName;
  if (!stageName) {
    throw new Error(`Couldn't find stage name.`);
  }
  return stageName;
}

export class BBHStage extends Stage {
  constructor(scope: Construct, id: string, props?: cdk.StageProps) {
    super(scope, id, props);

    new MainStack(this, 'MainStack');
  }
}
