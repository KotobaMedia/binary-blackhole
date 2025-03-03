import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';

import { API } from './main/api';
import { DDB } from './main/ddb';
import { VPC } from './main/vpc';
import { getStageName, getVpcId } from './stage';

export class MainStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // Get VPC ID from environment variables based on stage
    const vpcId = getVpcId(this);

    if (!vpcId) {
      throw new Error(`VPC ID not found. Please set the VPC_ID_${getStageName(this).toUpperCase()} environment variable.`);
    }

    const vpc = new VPC(this, 'VPC', { vpcId });
    const ddb = new DDB(this, 'DDB', {});
    const api = new API(this, 'API', {
      mainTable: ddb.mainTable,
      vpc: vpc.vpc,
      securityGroup: vpc.lambdaSecurityGroup
    });

    new cdk.CfnOutput(this, 'APIFnUrl', {
      value: api.apiFnUrl.url,
    });

    new cdk.CfnOutput(this, 'VpcId', {
      value: vpc.vpc.vpcId,
    });
  }
}
