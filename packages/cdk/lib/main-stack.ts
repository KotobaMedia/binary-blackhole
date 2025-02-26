import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';

import { API } from './main/api';
import { DDB } from './main/ddb';

export class MainStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const ddb = new DDB(this, 'DDB', {});
    const api = new API(this, 'API', {  mainTable: ddb.mainTable });

    new cdk.CfnOutput(this, 'APIFnUrl', {
      value: api.apiFnUrl.url,
    });

  }
}
