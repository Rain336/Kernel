from .task import TaskRegistry

def main():
    parser = TaskRegistry().build_argparse()
    args = parser.parse_args()

    task = TaskRegistry().create_task(args.action)
    task.extract_arguments(args)

    task.run()