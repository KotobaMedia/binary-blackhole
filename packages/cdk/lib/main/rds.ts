import * as ec2 from 'aws-cdk-lib/aws-ec2';
import { Construct } from 'constructs';
import * as rds from 'aws-cdk-lib/aws-rds';

type APIProps = {
  vpc: ec2.IVpc;
}

export class RDS extends Construct {
  cluster: rds.DatabaseCluster;
  securityGroup: ec2.SecurityGroup;

  constructor(scope: Construct, id: string, { vpc }: APIProps) {
    super(scope, id);

    this.securityGroup = new ec2.SecurityGroup(this, 'RDSSecurityGroup', {
      vpc,
      allowAllOutbound: true,
    });

    this.cluster = new rds.DatabaseCluster(this, 'AuroraSlsV2Cluster', {
      engine: rds.DatabaseClusterEngine.auroraPostgres({
        version: rds.AuroraPostgresEngineVersion.VER_16_4,
      }),
      vpc,
      defaultDatabaseName: 'initial_bbh',
      serverlessV2MinCapacity: 0.5,
      serverlessV2MaxCapacity: 16,
      writer: rds.ClusterInstance.serverlessV2('writer'),
      readers: [],
      vpcSubnets: {
        subnetType: ec2.SubnetType.PRIVATE_ISOLATED,
      },
      securityGroups: [this.securityGroup],
    });
  }
}
