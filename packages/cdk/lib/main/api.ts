import * as path from 'node:path';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as dynamodb from 'aws-cdk-lib/aws-dynamodb';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import { Construct } from 'constructs';
import { RustFunction } from 'cargo-lambda-cdk';
import { Duration } from 'aws-cdk-lib';
import { getStageName } from '../stage';

type APIProps = {
  mainTable: dynamodb.Table;
  vpc: ec2.IVpc;
  securityGroup: ec2.SecurityGroup;
}

export class API extends Construct {
  apiFn: RustFunction;
  apiFnUrl: lambda.FunctionUrl;

  constructor(scope: Construct, id: string, { mainTable, vpc, securityGroup }: APIProps) {
    super(scope, id);

    this.apiFn = new RustFunction(this, 'API', {
      binaryName: 'api',
      manifestPath: path.join(__dirname, '../../../../Cargo.toml'),
      architecture: lambda.Architecture.ARM_64,
      environment: {
        MAIN_TABLE: mainTable.tableName,
        POSTGRES_CONN_STR: process.env[`POSTGRES_CONN_STR_${getStageName(this)}`] ?? '',
      },
      memorySize: 256,
      timeout: Duration.seconds(30),
      vpc,
      vpcSubnets: {
        subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS,
      },
      securityGroups: [securityGroup],
    });

    mainTable.grantReadWriteData(this.apiFn);

    this.apiFnUrl = this.apiFn.addFunctionUrl({
      authType: lambda.FunctionUrlAuthType.NONE,
      cors: {
        allowedOrigins: ['*'],
      },
    });
  }
}
