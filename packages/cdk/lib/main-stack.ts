import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import { API } from './main/api';

export class MainStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const api = new API(this, 'API', {});

    new cdk.CfnOutput(this, 'APIFnUrl', {
      value: api.apiFnUrl.url,
    });

  }
}
