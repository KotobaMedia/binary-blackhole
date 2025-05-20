import { Construct } from "constructs";
import * as dynamodb from "aws-cdk-lib/aws-dynamodb";

export class DDB extends Construct {
  mainTable: dynamodb.Table;

  constructor(scope: Construct, id: string, props: {}) {
    super(scope, id);

    this.mainTable = new dynamodb.Table(this, "MainTable", {
      billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
      partitionKey: {
        name: "pk",
        type: dynamodb.AttributeType.STRING,
      },
      sortKey: {
        name: "sk",
        type: dynamodb.AttributeType.STRING,
      },
    });

    this.mainTable.addGlobalSecondaryIndex({
      indexName: "gsi1",
      partitionKey: {
        name: "gsi1pk",
        type: dynamodb.AttributeType.STRING,
      },
      projectionType: dynamodb.ProjectionType.ALL,
    });
  }
}
