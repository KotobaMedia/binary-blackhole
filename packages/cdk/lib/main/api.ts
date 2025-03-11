import * as path from 'node:path';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as dynamodb from 'aws-cdk-lib/aws-dynamodb';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as rds from 'aws-cdk-lib/aws-rds';
import { Construct } from 'constructs';
import { RustFunction } from 'cargo-lambda-cdk';
import { Duration } from 'aws-cdk-lib';
import { getStageName } from '../stage';

type APIProps = {
  mainTable: dynamodb.Table;
  vpc: ec2.IVpc;
  rds: rds.DatabaseCluster;
}

export class API extends Construct {
  apiFn: RustFunction;
  apiFnUrl: lambda.FunctionUrl;
  streamingFn: RustFunction;
  streamingFnUrl: lambda.FunctionUrl;
  securityGroup: ec2.SecurityGroup;

  constructor(scope: Construct, id: string, { mainTable, vpc, rds }: APIProps) {
    super(scope, id);

    this.securityGroup = new ec2.SecurityGroup(this, 'LambdaSecurityGroup', {
      vpc,
      allowAllOutbound: true,
      allowAllIpv6Outbound: true,
    });

    // TODO: Update to use IAM-based authentication?
    const clusterReadEndpoint = rds.clusterReadEndpoint.hostname;
    const clusterReadPort = rds.clusterReadEndpoint.port.toString();
    const rdsPassword = process.env[`RDS_PASSWORD_${getStageName(this)}`];
    const connStr = `host=${clusterReadEndpoint} port=${clusterReadPort} user=bbh_ro dbname=bbh password=${rdsPassword}`;

    this.apiFn = new RustFunction(this, 'API', {
      binaryName: 'api',
      manifestPath: path.join(__dirname, '../../../../Cargo.toml'),
      architecture: lambda.Architecture.ARM_64,
      environment: {
        TABLE_NAME: mainTable.tableName,
        POSTGRES_CONN_STR: connStr,
        OPENAI_API_KEY: process.env.OPENAI_API_KEY ?? '',
      },
      memorySize: 512,
      timeout: Duration.seconds(30),
      vpc,
      vpcSubnets: {
        subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS,
      },
      ipv6AllowedForDualStack: true,
      securityGroups: [this.securityGroup],
    });

    mainTable.grantReadWriteData(this.apiFn);

    this.apiFnUrl = this.apiFn.addFunctionUrl({
      authType: lambda.FunctionUrlAuthType.NONE,
    });

    this.streamingFn = new RustFunction(this, 'API', {
      binaryName: 'api-streaming',
      manifestPath: path.join(__dirname, '../../../../Cargo.toml'),
      architecture: lambda.Architecture.ARM_64,
      environment: {
        TABLE_NAME: mainTable.tableName,
        POSTGRES_CONN_STR: connStr,
        OPENAI_API_KEY: process.env.OPENAI_API_KEY ?? '',
      },
      memorySize: 512,
      timeout: Duration.seconds(30),
      vpc,
      vpcSubnets: {
        subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS,
      },
      ipv6AllowedForDualStack: true,
      securityGroups: [this.securityGroup],
    });

    mainTable.grantReadWriteData(this.streamingFn);

    this.streamingFnUrl = this.streamingFn.addFunctionUrl({
      authType: lambda.FunctionUrlAuthType.NONE,
      invokeMode: lambda.InvokeMode.RESPONSE_STREAM,
    });
  }
}
