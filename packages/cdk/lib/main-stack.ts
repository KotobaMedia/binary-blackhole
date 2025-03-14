import * as cdk from "aws-cdk-lib";
import * as ec2 from "aws-cdk-lib/aws-ec2";
import { Construct } from "constructs";

import { API } from "./main/api";
import { DDB } from "./main/ddb";
import { VPC } from "./main/network";
import { RDS } from "./main/rds";

export class MainStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const vpc = new VPC(this, "VPC", {});
    const rds = new RDS(this, "RDS", { vpc: vpc.vpc });
    const ddb = new DDB(this, "DDB", {});
    const api = new API(this, "API", {
      mainTable: ddb.mainTable,
      vpc: vpc.vpc,
      rds: rds.cluster,
    });
    rds.securityGroup.connections.allowFrom(
      api.securityGroup,
      ec2.Port.tcp(5432),
    );

    new cdk.CfnOutput(this, "APIFnUrl", {
      value: api.apiFnUrl.url,
    });
    new cdk.CfnOutput(this, "StreamingFnUrl", {
      value: api.streamingFnUrl.url,
    });

    new cdk.CfnOutput(this, "VpcId", {
      value: vpc.vpc.vpcId,
    });
  }
}
