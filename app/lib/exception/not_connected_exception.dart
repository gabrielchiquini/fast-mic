import 'default_exception.dart';

class NotConnectedException extends DefaultException {
  NotConnectedException() : super("Not connected to any network");
}
