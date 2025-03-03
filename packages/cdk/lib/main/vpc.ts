import * as ec2 from 'aws-cdk-lib/aws-ec2';
import { Construct } from 'constructs';

export interface VPCProps {
  vpcId: string; // Now required, no longer optional
}

export class VPC extends Construct {
  vpc: ec2.IVpc;
  lambdaSecurityGroup: ec2.SecurityGroup;

  constructor(scope: Construct, id: string, props: VPCProps) {
    super(scope, id);

    // Look up an existing VPC by ID (no fallback)
    this.vpc = ec2.Vpc.fromLookup(this, 'ExistingVpc', {
      vpcId: props.vpcId,
    });

    // Create security group for Lambda functions
    this.lambdaSecurityGroup = new ec2.SecurityGroup(this, 'LambdaSecurityGroup', {
      vpc: this.vpc,
      description: 'Security group for Lambda functions',
      allowAllOutbound: true,
    });
  }
}
