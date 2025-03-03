import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import { Stage } from 'aws-cdk-lib';
import { MainStack } from './main-stack';

export function getStageName(scope: Construct): string {
  const stackName = scope.node.path.split('/')[1];
  return stackName.split('-').pop() || 'dev';
}

export function getVpcId(scope: Construct): string | undefined {
  const stage = getStageName(scope);
  const vpcIdEnvVar = `VPC_ID_${stage.toUpperCase()}`;
  return process.env[vpcIdEnvVar];
}

export class BBHStage extends Stage {
  constructor(scope: Construct, id: string, props?: cdk.StageProps) {
    super(scope, id, props);

    new MainStack(this, 'MainStack');
  }
}
