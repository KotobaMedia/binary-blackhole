import * as path from 'node:path';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as dynamodb from 'aws-cdk-lib/aws-dynamodb';
import { Construct } from 'constructs';
import { RustFunction } from 'cargo-lambda-cdk';

type APIProps = {
  mainTable: dynamodb.Table;
}

export class API extends Construct {
  apiFn: RustFunction;
  apiFnUrl: lambda.FunctionUrl;

  constructor(scope: Construct, id: string, { mainTable }: APIProps) {
    super(scope, id);

    this.apiFn = new RustFunction(this, 'API', {
      binaryName: 'api',
      manifestPath: path.join(__dirname, '../../../../Cargo.toml'),
      architecture: lambda.Architecture.ARM_64,
      environment: {
        MAIN_TABLE: mainTable.tableName,
      },
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
